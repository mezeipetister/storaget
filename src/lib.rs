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

// Need this?
pub trait StorageHasID {
    fn get_id(&self) -> &str;
}

// Really?
pub trait StorageObject<'a> {
    fn load(&'a mut self, data: &str) -> StorageResult<()>;
}

pub struct Storage<T> {
    data: T,
    path: &'static str,
}

impl<'a, T: 'a> Storage<Vec<T>>
where
    T: StorageObject<'a> + StorageHasID,
{
    pub fn new(path: &'static str) -> Self {
        Storage {
            data: Vec::new(),
            path,
        }
    }
    pub fn get_by_id(&'a mut self, id: &str) -> StorageResult<DataObject<T>> {
        for item in &mut self.data {
            if item.get_id() == id {
                return Ok(DataObject {
                    data: item,
                    path: self.path,
                });
            }
        }
        Err(Error::InternalError(format!("ID: {} not found", id)))
    }
    // TODO: implement ID is unique check!
    pub fn add_to_storage(&'a mut self, new_object: T) -> StorageResult<()> {
        self.data.push(new_object);
        Ok(())
    }
}

impl<'a, T> Storage<T>
where
    T: StorageObject<'a>,
{
    fn say_hi(&self) {
        println!("Hi!");
    }
}

#[derive(Debug)]
pub struct DataObject<'a, T: 'a> {
    data: &'a mut T,
    path: &'static str,
}

impl<'a, T> DataObject<'a, T> {
    pub fn get(self) -> &'a T {
        self.data
    }
    pub fn get_mut(self) -> &'a mut T {
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
    use super::*;
    #[test]
    fn basic_test() {
        struct User {
            id: String,
            name: String,
        }
        impl User {
            fn new(id: &str, name: &str) -> Self {
                User {
                    id: id.into(),
                    name: name.into(),
                }
            }
            fn set_name(&mut self, name: &str) {
                self.name = name.into();
            }
        }
        impl<'a> StorageObject<'a> for User {
            fn load(&'a mut self, data: &str) -> StorageResult<()> {
                Ok(())
            }
        }
        impl StorageHasID for User {
            fn get_id(&self) -> &str {
                &self.id
            }
        }
        let mut storage: Storage<Vec<User>> = Storage::new("data");
        storage.add_to_storage(User::new("1", "Kriszti")).unwrap();
        storage.add_to_storage(User::new("2", "Peti")).unwrap();
        storage.add_to_storage(User::new("3", "Gabi")).unwrap();

        // let mut a = vec![1,2,3,4,5];
        // a.iter_mut();

        assert_eq!(storage.get_by_id("1").unwrap().get().name, "Kriszti");
        assert_eq!(storage.get_by_id("2").unwrap().get().name, "Peti");
        assert_eq!(storage.get_by_id("3").unwrap().get().name, "Gabi");
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
        assert_eq!(storage.get_by_id("3").unwrap().get().name, "Gabi!!!!");
        assert_eq!(storage.get_by_id("4").is_err(), true);

        let u1 = storage.get_by_id("1").unwrap().get();
        assert_eq!(u1.name, "Kriszti");

        if let Ok(u1) = storage.get_by_id("1") {
            assert_eq!(u1.get().name, "Kriszti");
        }

        if let Ok(mut u2) = storage.get_by_id("1") {
            u2.get_mut().set_name("Kriszti!");
        }
        assert_eq!(storage.get_by_id("1").unwrap().get().name, "Kriszti!");
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
