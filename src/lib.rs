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

#![feature(test)]

extern crate rand;
extern crate test;
pub mod prelude;
pub use prelude::*;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub trait StorageObject<'a> {
    fn load(&mut self, data: &str) -> StorageResult<()>;
}

pub struct Storage<T> {
    data: Mutex<Vec<Arc<Mutex<T>>>>,
    lookup_table: Mutex<BTreeMap<String, usize>>,
    path: &'static str,
}

impl<T> Storage<T>
where
    T: HasId,
{
    pub fn new(path: &'static str) -> Self {
        Storage {
            data: Mutex::new(Vec::new()),
            lookup_table: Mutex::new(BTreeMap::new()),
            path,
        }
    }
    pub fn add(&self, object: T) -> StorageResult<()> {
        let mut data = self.data.lock().unwrap();
        self.lookup_table
            .lock()
            .unwrap()
            .insert(object.get_id().to_owned(), data.len());
        data.push(Arc::new(Mutex::new(object)));
        Ok(())
    }
    pub fn get_by_id(&self, id: &str) -> StorageResult<DataObject<T>> {
        match self.lookup_table.lock().unwrap().get_key_value(id) {
            Some(r) => Ok(DataObject::new(
                self.data.lock().unwrap().get(*r.1).unwrap().clone(),
                self.path,
            )),
            None => Err(Error::ObjectNotFound),
        }
    }
    pub fn get_by_ids(&self, ids: &[&str]) -> Vec<(String, StorageResult<DataObject<T>>)> {
        let mut result: Vec<(String, StorageResult<DataObject<T>>)> = Vec::new();
        for id in ids {
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
}

impl<T> IntoIterator for &Storage<T>
where
    T: HasId + Clone,
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
    T: HasId,
{
    data: Vec<Arc<Mutex<T>>>,
    path: &'static str,
    index: usize,
}

impl<T> Iterator for DataIter<T>
where
    T: HasId,
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
    T: HasId,
{
    data: Arc<Mutex<T>>,
    path: &'static str,
}

impl<T> DataObject<T>
where
    T: HasId,
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
    pub fn update<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(&mut T) -> R,
    {
        println!("Saved!");
        f(&mut self.data.lock().unwrap())
    }
}

pub trait HasId {
    fn get_id(&self) -> &str;
}

#[derive(Clone)]
pub struct User {
    id: String,
    name: String,
    age: i32,
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
}

impl HasId for User {
    fn get_id(&self) -> &str {
        &self.id
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;
    use test::Bencher;
    // Generate random string with a given lenght.
    // Using english alphabet + integers + a few other basic characters.
    fn build_string(length: i32) -> String {
        let mut string = "".to_owned();
        let chars = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7',
            '8', '9', '_',
        ];
        for ch in 0..length {
            string.push(
                *chars
                    .get(rand::thread_rng().gen_range(0, chars.len()))
                    .unwrap(),
            );
        }
        string
    }
    #[test]
    fn test_storage_iter() {
        let storage = Storage::new("Data");
        let u1 = User::new("kriszta916", "Kriszti", 27);
        let u2 = User::new("purucka92", "Gabi", 27);
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add(u1).unwrap();
        storage.add(u2).unwrap();
        storage.add(u3).unwrap();
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
    }
    #[test]
    fn test_storage_add() {
        let storage = Storage::new("Data");
        let u1 = User::new("kriszta916", "Kriszti", 27);
        let u2 = User::new("purucka92", "Gabi", 27);
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add(u1).unwrap();
        storage.add(u2).unwrap();
        storage.add(u3).unwrap();
        let result = storage.get_by_id("purucka92").unwrap();
        assert_eq!(&result.get(|u| u.name.to_owned()), "Gabi");
        assert_eq!(&result.get(|u| u.get_name().to_owned()), "Gabi");
    }
    #[bench]
    fn bench_storage_data(b: &mut Bencher) {
        let storage = Storage::new("data");
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        b.iter(|| {
            let data = storage.data();
            println!("{}", data.len());
        });
    }
    #[bench]
    fn bench_storage_get_by_id(b: &mut Bencher) {
        let storage = Storage::new("data");
        for _ in 0..1000000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add(u3).unwrap();
        b.iter(|| {
            let result = storage.get_by_id("purucka92");
            println!("{}", result.is_ok());
        });
    }
    #[bench]
    fn bench_storage_iter_filter(b: &mut Bencher) {
        let storage = Storage::new("data");
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add(u3).unwrap();
        b.iter(|| {
            let result = storage
                .into_iter()
                .filter(|u| u.get(|u| u.get_id() == "mezeipetister"))
                .map(|u| u.get(|u| u.age))
                .collect::<Vec<i32>>();
            println!("{}", result.len());
        });
    }
    #[test]
    fn test_storage_iter_filter() {
        let storage = Storage::new("data");
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add(u3).unwrap();
        let result = storage
            .into_iter()
            .filter(|u| u.get(|u| u.get_id() == "mezeipetister"))
            .map(|u| u.get(|u| u.age))
            .collect::<Vec<i32>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap(), &31);
    }
    #[bench]
    fn bench_storage_iter_fold(b: &mut Bencher) {
        let storage = Storage::new("data");
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        b.iter(|| {
            let result = storage.into_iter().fold(0, |sum, u| sum + u.get(|u| u.age));
            println!("{}", result);
        });
    }
    #[bench]
    fn bench_storage_iter_fold_string_contains(b: &mut Bencher) {
        let storage = Storage::new("data");
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add(u3).unwrap();
        b.iter(|| {
            let result = storage
                .into_iter()
                .filter(|u| u.get(|u| u.get_name().contains("mezei")))
                .map(|u| u.get(|u| u.age))
                .collect::<Vec<i32>>();
            println!("{}", result.len());
        });
    }
    #[test]
    fn test_storage_iter_fold() {
        let storage = Storage::new("data");
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        let result = storage.into_iter().fold(0, |sum, u| sum + u.get(|u| u.age));
        assert_eq!(result > 1000, true);
    }
    #[test]
    fn test_storage_add_multiple() {
        let storage = Storage::new("data");
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        assert_eq!(storage.data.lock().unwrap().len(), 100000);
    }
    #[bench]
    fn bench_storage_add(b: &mut Bencher) {
        let storage = Storage::new("data");
        for _ in 0..100000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        b.iter(|| {
            storage
                .add(User::new(
                    &build_string(20),
                    &build_string(50),
                    rand::thread_rng().gen_range(10, 90),
                ))
                .unwrap();
        });
    }
    #[test]
    fn test_storage_get_by_id() {
        let storage = Storage::new("data");
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        let u3 = User::new("mezeipetister", "Peti", 31);
        storage.add(u3).unwrap();
        assert_eq!(storage.get_by_id("mezeipetister").is_ok(), true);
    }
    #[test]
    fn test_storage_get_by_ids() {
        let storage = Storage::new("data");
        for _ in 0..1000 {
            let u = User::new(
                &build_string(50),
                &build_string(100),
                rand::thread_rng().gen_range(10, 90),
            );
            storage.add(u).unwrap();
        }
        storage.add(User::new("demo1", "Demo 1", 9)).unwrap();
        storage.add(User::new("demo2", "Demo 2", 9)).unwrap();
        storage.add(User::new("demo3", "Demo 3", 9)).unwrap();
        assert_eq!(storage.get_by_ids(&["demo1", "demo2", "demo3"]).len(), 3);
        let result = storage.get_by_ids(&["demo1", "demo2", "demo3"]);
        for item in &result {
            assert_eq!(item.1.is_ok(), true);
        }
    }
}
