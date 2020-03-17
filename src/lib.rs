// The MIT License
// Copyright 2020 Peter Mezei <mezeipetister@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// Made with (L) from Hungary
// If you need any help please contact me
// at <mezeipetister@gmail.com>

#![feature(test)]

extern crate rand;
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

/// PackResult<T>
///
/// Generic Pack result type
/// contains Ok(T) or PackError
///
/// ```rust
/// use crate::*;
/// let res_ok: PackResult<i32> = Ok(32);
/// let res_err: PackResult<i32> = Err(PackError::ObjectNotFound);
/// ```
pub type PackResult<T> = Result<T, PackError>;

/// Pack Error type
/// For internal use
pub enum PackError {
    /// Any error that has a custom message.
    /// Any kind of error that has no other
    /// more specific variant in Error::*
    InternalError(String),
    /// Serialize Error
    /// error occured during serialiuation
    SerializeError(String),
    /// Deserialize Error
    /// error occured during deserialization
    DeserializeError(String),
    /// IO Error
    /// error during file operations
    IOError(String),
    /// Object not found in a storage.
    /// Usually using with get_by_id()
    ObjectNotFound,
    /// Path not found
    /// Using at reading data from path.
    PathNotFound,
}

// serde_yaml::Error to PackError
// implementation
impl From<serde_yaml::Error> for PackError {
    fn from(from: serde_yaml::Error) -> Self {
        PackError::SerializeError(from.to_string())
    }
}

// Well formatted display text for users
// TODO: Use error code and language translation
// for end-user error messages.
impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PackError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
            PackError::SerializeError(msg) => {
                write!(f, "Pack serialization error: {}", msg)
            }
            PackError::DeserializeError(msg) => {
                write!(f, "Pack deserialization error: {}", msg)
            }
            PackError::IOError(msg) => write!(f, "Pack IO error: {}", msg),
            PackError::PathNotFound => write!(f, "Path not found"),
            PackError::ObjectNotFound => {
                write!(f, "Storage object not found in storage.")
            }
        }
    }
}

// Well formatted debug text
// TODO: how to support localitation?
impl fmt::Debug for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
            PackError::SerializeError(msg) => {
                write!(f, "Pack serialization error: {}", msg)
            }
            PackError::DeserializeError(msg) => {
                write!(f, "Pack deserialization error: {}", msg)
            }
            PackError::IOError(msg) => write!(f, "Pack IO error: {}", msg),
            PackError::PathNotFound => write!(f, "Path not found"),
            PackError::ObjectNotFound => {
                write!(f, "Storage object not found in storage.")
            }
        }
    }
}

impl From<io::Error> for PackError {
    fn from(err: io::Error) -> Self {
        PackError::IOError(format!("{}", err))
    }
}

/// Pack<T>
/// Small FS layer around type T
/// Pack is responsible to sync T to the filesystem.
pub struct Pack<T>
where
    T: Serialize + Sized + Clone,
{
    data: T,
    path: &'static str,
}

/// PackGuard<'a, T>
/// Small mutable guard around type T
/// that implements Drop trait, and save T
/// to the filesystem when PackGuard is dropped.
///
/// Implements deref, deref_mut and drop
pub struct PackGuard<'a, T>
where
    T: Serialize + Sized + Clone,
{
    data: &'a mut T,
    path: &'static str,
}

/// VecPack<T>
/// Small FS layer around a Vec<Pack<T>>
/// The naming could be confusing a bit, as VecPack<T>
/// is rather FSLayer<Vec<Pack<T>>>, but maybe this could
/// be too long and unnecessary. So VecPack<T> behaves as
/// a special Vec<Pack<T>>.
pub struct VecPack<T>
where
    T: Serialize + Sized + Clone,
{
    data: Vec<Pack<T>>,
    path: &'static str,
}

// TODO: Should we rename it?
pub trait PackHasId<T>: Serialize {
    type Result;
    fn get_id(&self) -> &T;
}

impl<T> Pack<T>
where
    T: Serialize + Sized + Clone,
{
    /// Save Pack<T> manually
    /// to FS. Returns PackError if something
    /// wrong occures.
    pub fn save(&self) -> PackResult<()> {
        let mut buffer =
            BufWriter::new(File::create(&format!("{}.yml", &self.path))?);
        buffer.write_all(serde_yaml::to_string(&self.data)?.as_bytes())?;
        buffer.flush()?;
        Ok(())
    }
    /// Update Pack<T>
    /// Tries to update T, if SUCCESS
    /// then tries to save to FS, if SUCCESS
    /// returns R. If Fail, then doing data T
    /// rollback to backup, then return PackError.
    pub fn update<F, R>(&mut self, mut f: F) -> PackResult<R>
    where
        F: FnMut(&mut T) -> R,
    {
        // First clone data as a backup.
        let backup = self.data.clone();
        // Let's do the update process.
        let res = f(&mut self.data);
        // Try to save data to the FS
        match self.save() {
            // If success, then return the update result(s)
            Ok(_) => Ok(res),
            // If there is error occured during
            // saveing updated data
            Err(err) => {
                // Then rollback data to the backup.
                self.data = backup;
                // Return error
                Err(err)
            }
        }
    }
    /// Get(Fn) -> R
    /// Access data through closure
    /// Unmutable data access
    pub fn get<F, R>(&self, f: F) -> R
    where
        F: Fn(&T) -> R,
    {
        f(&self.data)
    }
    /// Map(Fn) -> R
    /// Syntactic sugar for Get(Fn) -> R
    pub fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&T) -> R,
    {
        f(&self.data)
    }
    /// as_mut() -> PackGuard<'a, T>
    /// returns
    pub fn as_mut(&mut self) -> PackGuard<'_, T> {
        PackGuard {
            data: &mut self.data,
            path: &self.path,
        }
    }
}

impl<T> Deref for Pack<T>
where
    T: Serialize + Sized + Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> Deref for PackGuard<'a, T>
where
    T: Serialize + Sized + Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for PackGuard<'a, T>
where
    T: Serialize + Sized + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T> Drop for PackGuard<'a, T>
where
    T: Serialize + Sized + Clone,
{
    fn drop(&mut self) {
        // TODO: Implement FS save!
        println!("PackGuard has dropped!");
    }
}
