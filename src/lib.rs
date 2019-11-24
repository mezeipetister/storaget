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

mod prelude;
pub use prelude::*;
use std::collections::BTreeMap;
use std::mem;

pub trait StorageMember<'a> {
    fn get_id(&self) -> &str;
}

pub struct Storage<T> {
    data: Vec<T>,
    lookup_table: BTreeMap<String, usize>,
    path: &'static str,
}

impl<'a, T: 'a> Storage<T>
where
    T: StorageMember<'a>,
{
    pub fn new(path: &'static str) -> Self {
        Storage {
            data: Vec::new(),
            lookup_table: BTreeMap::new(),
            path,
        }
    }
    pub fn get_by_id(&'a mut self, id: &str) -> StorageResult<DataObject<T>> {
        match self.lookup_table.get_key_value(id) {
            Some(r) => Ok(DataObject {
                data: self.data.get_mut(*r.1).unwrap(),
                path: self.path,
            }),
            None => Err(Error::InternalError(format!("ID: {} not found", id))),
        }
    }
    // TODO: implement ID is unique check!
    pub fn add_to_storage(&'a mut self, new_object: T) -> StorageResult<()> {
        self.lookup_table
            .insert(new_object.get_id().into(), self.data.len());
        self.data.push(new_object);
        Ok(())
    }
}

impl<'a, T> IntoIterator for &'a mut Storage<T> {
    type Item = DataObject<'a, T>;
    type IntoIter = DataIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        DataIter {
            data: &mut self.data,
            path: self.path,
        }
    }
}

pub struct DataIter<'a, T> {
    data: &'a mut [T],
    path: &'static str,
}

impl<'a, T> Iterator for DataIter<'a, T> {
    type Item = DataObject<'a, T>;
    fn next(&mut self) -> Option<DataObject<'a, T>> {
        let slice = mem::replace(&mut self.data, &mut []);
        match slice.split_first_mut() {
            Some((head, tail)) => {
                self.data = tail;
                Some(DataObject {
                    data: head,
                    path: self.path,
                })
            }
            None => None,
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct DataObject<'a, T: 'a> {
    data: &'a mut T,
    path: &'static str,
}

impl<'a, T> DataObject<'a, T> {
    pub fn get_ref(self) -> &'a T {
        self.data
    }
    #[deprecated(
        since = "0.1.0",
        note = "Deprecated function, just for internal test purpose. Please use update instead."
    )]
    pub fn get_ref_mut(self) -> &'a mut T {
        self.data
    }
    pub fn update<F: 'a, R>(&'a mut self, mut f: F) -> R
    where
        F: FnMut(&mut T) -> R,
    {
        f(self.data)
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate test;
    use super::*;
    use rand::prelude::*;
    use test::Bencher;
    // Demo user struct
    // We are going to use this in all the internal tests.
    struct User {
        id: String,
        name: String,
        age: i32,
        description: String,
        favorit_numbers: Vec<i32>,
        favorit_colors: Vec<String>,
    }
    impl User {
        // Construct new user
        fn new(id: &str, name: &str, age: i32) -> Self {
            User {
                id: id.into(),
                name: name.into(),
                age,
                description: "".into(),
                favorit_numbers: Vec::new(),
                favorit_colors: Vec::new(),
            }
        }
        // Get name
        fn get_name(&self) -> &str {
            &self.name
        }
        // Set name
        fn set_name(&mut self, name: &str) {
            self.name = name.into();
        }
    }
    impl<'a> StorageMember<'a> for User {
        fn get_id(&self) -> &str {
            &self.id
        }
    }
    // Generate random string with a given lenght.
    // Using english alphabet + integers + a few other basic characters.
    fn build_string(length: i32) -> String {
        let mut string = "".to_owned();
        let chars = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7',
            '8', '9', ' ', '_', '-', '.', '*',
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
    fn test_build_string() {
        let res = build_string(10);
        assert_eq!(res.len(), 10);
    }
    // Storage builder for tests.
    fn build_user_storage_of_dummies(size_of_storage: i32) -> Storage<User> {
        let mut storage: Storage<User> = Storage::new("data");
        for index in 0..size_of_storage {
            let mut user = User::new(
                &format!("{}", index),
                &build_string(20),
                rand::thread_rng().gen_range(10, 70),
            );
            user.description = build_string(50);
            user.favorit_colors = vec![build_string(5), build_string(10), build_string(7)];
            user.favorit_numbers = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
            storage.add_to_storage(user).unwrap();
        }
        storage
    }
    #[bench]
    fn bench_storage_add(b: &mut Bencher) {
        let mut storage = build_user_storage_of_dummies(100000);
        b.iter(|| {
            let _ = storage.add_to_storage(User::new(
                &format!("{}", rand::thread_rng().gen_range(1000, 1000000)),
                "Demo bench",
                12,
            ));
        })
    }
    #[bench]
    fn bench_storage_get_by_id(b: &mut Bencher) {
        let mut storage = build_user_storage_of_dummies(100000);
        let _ = storage.add_to_storage(User::new("demo_id", "Demo bench", 12));
        b.iter(|| {
            let _ = storage.get_by_id("demo_id");
        })
    }
    #[bench]
    fn bench_storage_update_name(b: &mut Bencher) {
        let mut storage = build_user_storage_of_dummies(100000);
        b.iter(|| {
            for item in &mut storage {
                item.update(|u| u.set_name("Demo modified name");
            }
        })
    }
    #[bench]
    fn bench_storage_filter(b: &mut Bencher) {
        let mut storage = build_user_storage_of_dummies(100000);
        let mut u1 = User::new("demo_id_iter1", "Lorem Ipsum", 9);
        u1.description = "Lorem Ipsum".to_owned();
        let mut u2 = User::new("demo_id_iter2", "Lorem Demo Item", 9);
        u2.description = "Lorem Demo Item".to_owned();
        storage.add_to_storage(u1).unwrap();
        storage.add_to_storage(u2).unwrap();
        b.iter(|| {
            &storage
                .into_iter()
                .filter(|x| (*x).data.description.contains("Lorem"))
                .map(|x| (x).data.name.to_owned())
                .collect::<Vec<String>>();
        })
    }
    pub fn demo(i: i32) {
        println!("{}", i);
    }
    #[bench]
    fn bench_storage_filter_fold_i32(b: &mut Bencher) {
        let mut storage = build_user_storage_of_dummies(1000000);
        let mut u1 = User::new("demo_id_iter1", "Lorem Ipsum", 9);
        u1.description = "Lorem Ipsum".to_owned();
        let mut u2 = User::new("demo_id_iter2", "Lorem Demo Item", 9);
        u2.description = "Lorem Demo Item".to_owned();
        storage.add_to_storage(u1).unwrap();
        storage.add_to_storage(u2).unwrap();
        b.iter(|| {
            let x = storage.into_iter().fold(0, |acc, u| acc + (u).data.age);
            demo(x);
        })
    }
    #[bench]
    fn bench_storage_filter_foreach(b: &mut Bencher) {
        let mut storage = build_user_storage_of_dummies(1000000);
        let mut u1 = User::new("demo_id_iter1", "Lorem Ipsum", 9);
        u1.description = "Lorem Ipsum".to_owned();
        let mut u2 = User::new("demo_id_iter2", "Lorem Demo Item", 9);
        u2.description = "Lorem Demo Item".to_owned();
        storage.add_to_storage(u1).unwrap();
        storage.add_to_storage(u2).unwrap();
        b.iter(|| {
            let mut age_total = 0;
            let mut age_max = 0;
            let mut age_min = 100;
            storage.into_iter().for_each(|u| {
                age_total += u.data.age;
                if u.data.age > age_max {
                    age_max = u.data.age;
                }
                if u.data.age < age_min {
                    age_min = u.data.age;
                }
            });
            println!("{}", age_total);
            println!("{}", age_max);
            println!("{}", age_min);
        })
    }
    #[bench]
    fn bench_storage_filter_fold_i32_average(b: &mut Bencher) {
        let mut storage = build_user_storage_of_dummies(100000);
        b.iter(|| {
            let a = storage.into_iter().fold(0, |acc, u| acc + (u).data.age)
                / storage.into_iter().data.len() as i32;
            println!("{}", a);
        })
    }
    #[test]
    fn test_storage_filter_fold_i32() {
        let mut storage = build_user_storage_of_dummies(100);
        assert_eq!(
            storage.into_iter().fold(0, |acc, u| acc + (u).data.age) > 1000,
            true
        );
    }
    #[test]
    fn test_storage_iter() {
        let mut storage = build_user_storage_of_dummies(1000);
        let mut u1 = User::new("demo_id_iter1", "Lorem Ipsum", 9);
        u1.description = "Lorem Ipsum".to_owned();
        let mut u2 = User::new("demo_id_iter2", "Lorem Demo Item", 9);
        u2.description = "Lorem Demo Item".to_owned();
        storage.add_to_storage(u1).unwrap();
        storage.add_to_storage(u2).unwrap();
        let res = &storage
            .into_iter()
            .filter(|x| (*x).data.name.contains("Lorem"))
            .map(|x| (x).data.name.to_owned())
            .collect::<Vec<String>>();
        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap(), "Lorem Ipsum");
        assert_eq!(res.get(1).unwrap(), "Lorem Demo Item");
    }
    #[test]
    fn test_storage_id() {
        impl<'a> StorageMember<'a> for i32 {
            fn get_id(&self) -> &str {
                let res: &'static str = "a";
                res
            }
        }
        assert_eq!(3.get_id(), "a");
    }
    #[test]
    fn basic_test() {
        let mut storage: Storage<User> = Storage::new("data");
        storage
            .add_to_storage(User::new("1", "Kriszti", 27))
            .unwrap();
        storage.add_to_storage(User::new("2", "Peti", 31)).unwrap();
        storage.add_to_storage(User::new("3", "Gabi", 27)).unwrap();

        // let mut a = vec![1,2,3,4,5];
        // a.iter_mut();

        assert_eq!(storage.get_by_id("1").unwrap().get_ref().name, "Kriszti");
        if let Ok(user) = storage.get_by_id("1") {
            assert_eq!(user.get_ref().name, "Kriszti");
        }
        let u1 = storage.get_by_id("1").unwrap();
        assert_eq!(u1.get_ref().name, "Kriszti");

        assert_eq!(storage.get_by_id("2").unwrap().get_ref().name, "Peti");
        assert_eq!(storage.get_by_id("3").unwrap().get_ref().name, "Gabi");

        storage
            .get_by_id("3")
            .unwrap()
            .update(|u| u.set_name("Gabi!"));
        let res = storage.get_by_id("3").unwrap().update(|u| -> bool {
            u.set_name("Gabi!!!!");
            true
        });
        assert_eq!(res, true);
        // Demo result type alias
        pub type DemoResult<T> = Result<T, ErrorI>;

        pub enum ErrorI {
            Error,
        }

        // Well formatted display text for users
        // TODO: Use error code and language translation for end-user error messages.
        impl std::fmt::Display for ErrorI {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    ErrorI::Error => write!(f, "Internal error"),
                }
            }
        }

        impl std::fmt::Debug for ErrorI {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    ErrorI::Error => write!(f, "Internal error!"),
                }
            }
        }

        let res = storage
            .get_by_id("3")
            .unwrap()
            .update(|u| -> DemoResult<()> {
                u.set_name("Gabi!!!!");
                Ok(())
            });
        assert_eq!(res.is_ok(), true);
        assert_eq!(storage.get_by_id("3").unwrap().get_ref().name, "Gabi!!!!");
        assert_eq!(storage.get_by_id("4").is_err(), true);

        let u1 = storage.get_by_id("1").unwrap().get_ref();
        assert_eq!(u1.name, "Kriszti");

        if let Ok(u1) = storage.get_by_id("1") {
            assert_eq!(u1.get_ref().name, "Kriszti");
        }

        if let Ok(u2) = storage.get_by_id("1") {
            u2.update(|u| u.set_name("Kriszti!"));
        }
        assert_eq!(storage.get_by_id("1").unwrap().get_ref().name, "Kriszti!");

        for mut user in &mut storage {
            user.update(|u| u.set_name("Bruhaha"));
        }

        assert_eq!(storage.get_by_id("1").unwrap().get_ref().name, "Bruhaha");
        assert_eq!(storage.get_by_id("2").unwrap().get_ref().name, "Bruhaha");
        assert_eq!(storage.get_by_id("3").unwrap().get_ref().name, "Bruhaha");
    }
}

// pub struct Storage<T> {
//     data: T,
//     path: &'static str,
// }

// pub trait StorageObject {}

// pub trait LoadFrom<T> {
//     fn load_from(path: &str) -> StorageResult<T>;
// }

// impl<T> LoadFrom<T> for T
// where
//     T: StorageObject,
// {
//     fn load_from(path: &str) -> StorageResult<T> {
//         Err(Error::InternalError("oo".into()))
//     }
// }

// impl<T, U> LoadFrom<U> for Vec<T>
// where
//     T: StorageObject,
// {
//     fn load_from(path: &str) -> StorageResult<U> {
//         Err(Error::InternalError("oo".into()))
//     }
// }

// impl<T> Storage<T>
// where
//     T: LoadFrom<T>,
// {
//     pub fn load_from(path: &'static str) -> StorageResult<Storage<T>> {
//         Ok(Storage {
//             data: T::load_from(path)?,
//             path,
//         })
//     }
// }
