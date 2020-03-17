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
use std::io;
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
            _ => write!(f, "Unknown error"),
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
            _ => write!(f, "Unknown error"),
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
    pub fn save(&self) -> PackResult<()> {
        Ok(())
    }
    pub fn update<F, R>(&mut self, mut f: F) -> R
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
            Ok(_) => res,
            // If there is error occured during
            // saveing updated data
            Err(err) => {
                // Then rollback data to the backup.
                self.data = backup;
                // Return error
                err
            }
        }
    }
}

impl<T> Pack<T>
where
    T: Serialize + Sized,
{
    /**
     * Load storage objects from path or create path if it does not exist
     */
    pub fn load_or_init<U>(path: &'static str) -> StorageResult<Self>
    where
        for<'de> T: Deserialize<'de>,
    {
        Ok(load_storage(path)?)
    }
    pub fn get_by_id(&self, id: &str) -> StorageResult<DataObject<T>> {
        let id = id.trim();
        match self.lookup_table.lock().unwrap().get_key_value(id) {
            Some(r) => Ok(DataObject::new(
                self.data.lock().unwrap().get(*r.1).unwrap().clone(),
                self.path,
            )),
            None => Err(Error::ObjectNotFound),
        }
    }
    pub fn get_by_ids(
        &self,
        ids: &[&str],
    ) -> Vec<(String, StorageResult<DataObject<T>>)> {
        let mut result: Vec<(String, StorageResult<DataObject<T>>)> =
            Vec::new();
        for id in ids {
            let id = id.trim();
            result.push(((*id).into(), self.get_by_id(id)));
        }
        result
    }
    pub fn data(&self) -> Vec<Arc<Mutex<T>>> {
        (&*self.data.lock().unwrap())
            .into_iter()
            .map(|v| v.clone())
            .collect::<Vec<Arc<Mutex<T>>>>()
    }
    pub fn into_data_objects(&self) -> Vec<DataObject<T>> {
        (&*self.data.lock().unwrap())
            .into_iter()
            .map(|v| DataObject::new(v.clone(), self.path))
            .collect::<Vec<DataObject<T>>>()
    }
    pub fn remove(&self) -> StorageResult<()> {
        remove_path(self.path)
    }
}

impl<T> IntoIterator for &Storage<T>
where
    T: StorageObject<ResultType = T>,
{
    type Item = DataObject<T>;
    type IntoIter = DataIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        DataIter {
            data: self.data(),
            path: self.path,
            index: 0,
        }
    }
}

pub struct DataIter<T>
where
    T: StorageObject,
{
    data: Vec<Arc<Mutex<T>>>,
    path: &'static str,
    index: usize,
}

impl<T> Iterator for DataIter<T>
where
    T: StorageObject,
{
    type Item = DataObject<T>;
    fn next(&mut self) -> Option<DataObject<T>> {
        match &self.data.get(self.index) {
            Some(item) => {
                self.index += 1;
                return Some(DataObject::new((*item).clone(), self.path));
            }
            None => return None,
        }
    }
}

pub struct DataObject<T>
where
    T: StorageObject,
{
    data: Arc<Mutex<T>>,
    path: &'static str,
}

impl<T> DataObject<T>
where
    T: StorageObject,
{
    fn new(data: Arc<Mutex<T>>, path: &'static str) -> Self {
        DataObject { data, path }
    }
    pub fn get<F, R>(&self, f: F) -> R
    where
        F: Fn(&T) -> R,
    {
        f(&self.data.lock().unwrap())
    }
    pub fn exec<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&T) -> R,
    {
        f(&self.data.lock().unwrap())
    }
    pub fn update<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&mut T) -> R,
    {
        let res = f(&mut self.data.lock().unwrap());
        save_storage_object(&*self.data.lock().unwrap(), self.path).unwrap();
        res
    }
    pub fn clone_data(&self) -> T {
        self.data.lock().unwrap().clone()
    }
    pub fn get_data_ref(&self) -> MutexGuard<T> {
        self.data.lock().unwrap()
    }
}

pub trait HasId {
    fn get_id(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use test::Bencher;

    // Dummy struct for tests.
    #[derive(Serialize, Deserialize, Clone)]
    pub struct User0 {
        old_id: String,
        old_name: String,
        old_age: i32,
    }
    // Dummy struct for tests.
    #[derive(Serialize, Deserialize, Clone)]
    pub struct User {
        id: String,
        name: String,
        age: i32,
    }

    impl StorageObject for User0 {
        type ResultType = User0;
        fn get_id(&self) -> &str {
            &self.old_id
        }
        fn try_from(from: &str) -> StorageResult<Self::ResultType> {
            match deserialize_object(from) {
                Ok(res) => Ok(res),
                Err(_) => Err(Error::DeserializeError("Ooo".to_string())),
            }
        }
    }

    impl User {
        pub fn new(id: &str, name: &str, age: i32) -> Self {
            User {
                id: id.into(),
                name: name.into(),
                age,
            }
        }
        pub fn get_name(&self) -> &str {
            &self.name
        }
        pub fn get_age(&self) -> i32 {
            self.age
        }
    }

    impl StorageObject for User {
        type ResultType = User;
        fn get_id(&self) -> &str {
            &self.id
        }
        fn try_from(from: &str) -> StorageResult<Self::ResultType> {
            match deserialize_object(from) {
                Ok(res) => Ok(res),
                Err(_) => Ok(User0::try_from(from)?.into()),
            }
        }
    }

    impl From<User0> for User {
        fn from(from: User0) -> Self {
            User {
                id: from.old_id,
                name: from.old_name,
                age: from.old_age,
            }
        }
    }

    // Generate random string with a given lenght.
    // Using english alphabet + integers + a few other basic characters.
    fn build_string(length: i32) -> String {
        let mut string = "".to_owned();
        let chars = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
            'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_',
        ];
        for _ in 0..length {
            string.push(
                *chars
                    .get(rand::thread_rng().gen_range(0, chars.len()))
                    .unwrap(),
            );
        }
        string
    }
    #[test]
    fn test_try_from() {
        let old_storage =
            Storage::load_or_init::<User0>("data/testasd").unwrap();
        let user = User0 {
            old_id: "1".to_string(),
            old_name: "Demo".to_string(),
            old_age: 31,
        };
        old_storage.add_to_storage(user).unwrap();
        let new_storage: Storage<User> =
            Storage::load_or_init::<User>("data/testasd").unwrap();
        let user_name = match new_storage.get_by_id("1") {
            Ok(res) => res.get(|u| u.get_name().to_string()),
            Err(_) => "none".to_string(),
        };
        assert_eq!(user_name, "Demo");
        // new_storage.remove().unwrap();
    }
    #[test]
    fn test_storage_iter() {
        let storage = Storage::load_or_init::<User>("data/test9").unwrap();
        let u1 = User::new("kriszta916", "Kriszti", 27);
        let u2 = User::new("purucka92", "Gabi", 27);
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add_to_storage(u1).unwrap();
        storage.add_to_storage(u2).unwrap();
        storage.add_to_storage(u3).unwrap();
        let mut index = 0;
        for _ in &storage {
            index += 1;
        }
        assert_eq!(index, 3);
        let result = &storage
            .into_iter()
            .filter(|u| u.get(|u| u.age < 31))
            .map(|u| u.get(|u| u.age))
            .collect::<Vec<i32>>();
        assert_eq!(result.len(), 2);
        storage.remove().unwrap();
    }
    #[test]
    fn test_storage_add() {
        let storage = Storage::load_or_init::<User>("data/test10").unwrap();
        let u1 = User::new("kriszta916", "Kriszti", 27);
        let u2 = User::new("purucka92", "Gabi", 27);
        let u3 = User::new("mezeipetister", "Peti", 31);
        assert_eq!(storage.add_to_storage(u1).is_ok(), true);
        assert_eq!(storage.add_to_storage(u2).is_ok(), true);
        assert_eq!(storage.add_to_storage(u3.clone()).is_ok(), true);
        assert_eq!(storage.add_to_storage(u3.clone()).is_ok(), false);
        assert_eq!(storage.add_to_storage(u3.clone()).is_ok(), false);
        let result = storage.get_by_id("purucka92").unwrap();
        assert_eq!(&result.get(|u| u.name.to_owned()), "Gabi");
        assert_eq!(&result.get(|u| u.get_name().to_owned()), "Gabi");
        storage.remove().unwrap();
    }
    #[bench]
    fn bench_storage_data(b: &mut Bencher) {
        let storage = Storage::load_or_init::<User>("data/test10").unwrap();
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        b.iter(|| {
            let data = storage.data();
            println!("{}", data.len());
        });
        storage.remove().unwrap();
    }
    #[bench]
    fn bench_storage_get_by_id(b: &mut Bencher) {
        let storage = Storage::load_or_init::<User>("data/test11").unwrap();
        for _ in 0..1000000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add_to_storage(u3).unwrap();
        b.iter(|| {
            let result = storage.get_by_id("purucka92");
            println!("{}", result.is_ok());
        });
        storage.remove().unwrap();
    }
    #[bench]
    fn bench_storage_iter_filter(b: &mut Bencher) {
        let storage = Storage::load_or_init::<User>("dataTtest12").unwrap();
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add_to_storage(u3).unwrap();
        b.iter(|| {
            let result = storage
                .into_iter()
                .filter(|u| u.get(|u| u.get_id() == "mezeipetister"))
                .map(|u| u.get(|u| u.age))
                .collect::<Vec<i32>>();
            println!("{}", result.len());
        });
        storage.remove().unwrap();
    }
    #[test]
    fn test_storage_iter_filter() {
        let storage = Storage::load_or_init::<User>("data/test13").unwrap();
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add_to_storage(u3).unwrap();
        let result = storage
            .into_iter()
            .filter(|u| u.get(|u| u.get_id() == "mezeipetister"))
            .map(|u| u.get(|u| u.age))
            .collect::<Vec<i32>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap(), &31);
        storage.remove().unwrap();
    }
    #[bench]
    fn bench_storage_iter_fold(b: &mut Bencher) {
        let storage = Storage::load_or_init::<User>("data/test14").unwrap();
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        b.iter(|| {
            let result =
                storage.into_iter().fold(0, |sum, u| sum + u.get(|u| u.age));
            println!("{}", result);
        });
        storage.remove().unwrap();
    }
    #[bench]
    fn bench_storage_iter_fold_string_contains(b: &mut Bencher) {
        let storage = Storage::load_or_init::<User>("data/test15").unwrap();
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add_to_storage(u3).unwrap();
        b.iter(|| {
            let result = storage
                .into_iter()
                .filter(|u| u.get(|u| u.get_name().contains("mezei")))
                .map(|u| u.get(|u| u.age))
                .collect::<Vec<i32>>();
            println!("{}", result.len());
        });
        storage.remove().unwrap();
    }
    #[test]
    fn test_storage_iter_fold() {
        let storage = Storage::load_or_init::<User>("data/test16").unwrap();
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        let result =
            storage.into_iter().fold(0, |sum, u| sum + u.get(|u| u.age));
        assert_eq!(result > 1000, true);
        storage.remove().unwrap();
    }
    #[test]
    fn test_storage_iter_exec() {
        let storage = Storage::load_or_init::<User>("data/test67").unwrap();
        for _ in 0..1000 {
            let u = User::new(&build_string(50), &build_string(100), 1);
            storage.add_to_storage(u).unwrap();
        }
        let mut i = 0;
        storage.into_iter().for_each(|u| {
            u.exec(|u| {
                i += u.get_age();
            })
        });
        assert_eq!(i == 1000, true);
        storage.remove().unwrap();
    }
    #[test]
    fn test_storage_add_multiple() {
        let storage = Storage::load_or_init::<User>("data/test17").unwrap();
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        assert_eq!(storage.data.lock().unwrap().len(), 1000);
        storage.remove().unwrap();
    }
    #[bench]
    fn bench_storage_add(b: &mut Bencher) {
        let storage = Storage::load_or_init::<User>("data/test18").unwrap();
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        b.iter(|| {
            storage
                .add_to_storage(User::new(
                    &build_string(20),
                    &build_string(50),
                    rand::thread_rng().gen_range(10, 90),
                ))
                .unwrap();
        });
        storage.remove().unwrap();
    }
    #[test]
    fn test_storage_get_by_id() {
        let storage = Storage::load_or_init::<User>("data/test19").unwrap();
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add_to_storage(u3).unwrap();
        assert_eq!(storage.get_by_id("mezeipetister").is_ok(), true);
        storage.remove().unwrap();
    }
    #[test]
    fn test_get_data_ref() {
        let storage = Storage::load_or_init::<User>("data/test192").unwrap();
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add_to_storage(u3).unwrap();
        let data = storage.get_by_id("mezeipetister").unwrap();
        let d = data.get_data_ref();
        assert_eq!(d.get_id(), "mezeipetister");
        assert_eq!(d.get_age(), 31);
        storage.remove().unwrap();
    }
    #[test]
    fn test_storage_get_by_ids() {
        let storage = Storage::load_or_init::<User>("data/test20").unwrap();
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add_to_storage(u).unwrap();
        }
        storage
            .add_to_storage(User::new("demo1", "Demo 1", 9))
            .unwrap();
        storage
            .add_to_storage(User::new("demo2", "Demo 2", 9))
            .unwrap();
        storage
            .add_to_storage(User::new("demo3", "Demo 3", 9))
            .unwrap();
        assert_eq!(storage.get_by_ids(&["demo1", "demo2", "demo3"]).len(), 3);
        let result = storage.get_by_ids(&["demo1", "demo2", "demo3"]);
        for item in &result {
            assert_eq!(item.1.is_ok(), true);
        }
        storage.remove().unwrap();
    }
}
