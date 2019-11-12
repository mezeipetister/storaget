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

mod prelude;
pub use prelude::*;

pub struct Storage<T> {
    data: T,
    path: &'static str,
}

pub trait StorageObject {}

pub trait LoadFrom<T> {
    fn load_from(path: &str) -> StorageResult<T>;
}

impl<T> LoadFrom<T> for T
where
    T: StorageObject,
{
    fn load_from(path: &str) -> StorageResult<T> {
        Err(Error::InternalError("oo".into()))
    }
}

impl<T, U> LoadFrom<U> for Vec<T>
where
    T: StorageObject,
{
    fn load_from(path: &str) -> StorageResult<U> {
        Err(Error::InternalError("oo".into()))
    }
}

impl<T> Storage<T>
where
    T: LoadFrom<T>,
{
    pub fn load_from(path: &'static str) -> StorageResult<Storage<T>> {
        Ok(Storage {
            data: T::load_from(path)?,
            path,
        })
    }
}

#[test]
fn simple_test() {
    struct User {
        id: String,
        name: String,
    }
    impl StorageObject for u32 {}
    impl StorageObject for User {}
    let users: Storage<Vec<User>> = Storage::load_from("data").unwrap();
    let user: Storage<User> = Storage::load_from("data").unwrap();
}
