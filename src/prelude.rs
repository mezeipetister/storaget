// Copyright (C) 2019 Peter Mezei
//
// This file is part of Project A.
//
// Project A is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// (at your option) any later version.
//
// Project A is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Project A.  If not, see <http://www.gnu.org/licenses/>.

use std::convert::From;
use std::fmt;
use std::io;

pub type StorageResult<T> = Result<T, Error>;

/// Storage Error type
/// For internal use
pub enum Error {
    /// Any error that has a custom message.
    /// Any kind of error that has no other
    /// more specific variant in Error::*
    InternalError(String),
    /// Object not found in a storage.
    /// Usually using with get_by_id()
    ObjectNotFound,
    /// Path not found
    /// Using at reading data from path.
    PathNotFound,
    SerializeError(String),
    DeserializeError(String),
    IOError(String),
}

// Well formatted display text for users
// TODO: Use error code and language translation for end-user error messages.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
            Error::ObjectNotFound => write!(f, "Storage object not found in storage."),
            Error::PathNotFound => write!(f, "Path not found"),
            _ => write!(f, "Unknown error"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
            Error::ObjectNotFound => write!(f, "Storage object not found in storage."),
            Error::PathNotFound => write!(f, "Path not found"),
            _ => write!(f, "Unknown error"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(format!("{}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display() {
        let err = Error::InternalError("test error".into());
        assert_eq!(format!("{}", err), "Internal error: test error");
    }
    #[test]
    fn test_debug() {
        let err = Error::InternalError("test error".into());
        assert_eq!(format!("{:?}", err), "Internal error: test error");
    }
    #[test]
    fn test_error() {
        let e = Error::InternalError("test".into());
        assert_eq!(format!("{}", e), "Internal error: test");
        assert_eq!(format!("{:?}", e), "Internal error: test");
        let f = Error::ObjectNotFound;
        assert_eq!(format!("{}", f), "Storage object not found in storage.");
        assert_eq!(format!("{:?}", f), "Storage object not found in storage.");
        let g = Error::PathNotFound;
        assert_eq!(format!("{}", g), "Path not found");
        assert_eq!(format!("{:?}", g), "Path not found");
    }
}
