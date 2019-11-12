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
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub type AppResult<T> = Result<T, Error>;

pub enum Error {
    InternalError(String),
}

use Error::*;

// Well formatted display text for users
// TODO: Use error code and language translation for end-user error messages.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

/**
 * Storage DESIGN
 *
 * Functions:
 *  - pub fn load_storage(path: &'str) -> Result<Vec<T>, String>
 *  - pub fn add_to_storage(storage: &Storage, object: StorageObject) -> Result<Ok(&StorageObject), String>
 *  -
 *  - Serialize    -|_____ Use these methods in loading
 *  - Deserialize  -|      and StorageObject.save() method
 */

/// # StorageObject
///
/// Storage can hold any StorageObject<T>.
/// Storage object ensures that an object can save and reload itself.
pub trait StorageObject: Serialize + Sized {
    fn get_id(&self) -> &str;
    fn reload(&mut self) -> AppResult<()>;
    fn get_path(&self) -> Option<&str>;
    fn set_path(&mut self, path: &str) -> AppResult<()>;
    fn get_date_created(&self) -> DateTime<Utc>;
    // Save autoimplementation
    fn save(&self) -> AppResult<()>
    where
        Self: Serialize + Sized,
    {
        save_storage_object(self)
    }
    // Update auto implementation
    fn update<F>(&mut self, mut f: F) -> AppResult<()>
    where
        F: FnMut(&mut Self) -> AppResult<()>,
    {
        f(self)?;
        self.save()?;
        Ok(())
    }
}

/// # Storage\<T\> experimental FS in-memory storage
pub struct Storage<T> {
    path: &'static str,
    pub data: Vec<T>,
}

impl<T> Storage<T> {
    // TODO: Doc comment + usage!
    pub fn remove(&self) -> bool {
        if Path::new(&self.path).exists() {
            match fs::remove_dir_all(&self.path) {
                Ok(_) => return true,
                Err(_) => return false,
            }
        }
        false
    }
}

/// # Load storage objects from path
///
/// Load storage objects from path
/// If path does not exist, create it.
/// During object loading, try to:
///  1) serialize objects
///  2) checking schema version
///  3) try schema update if it's needed.
///
/// *We use turbofish style*
pub fn load_storage<'a, T>(path: &'static str) -> AppResult<Storage<T>>
where
    for<'de> T: Deserialize<'de> + 'a + StorageObject,
{
    let mut storage: Storage<T> = Storage {
        path,
        data: Vec::new(),
    };
    if !Path::new(path).exists() {
        match fs::create_dir_all(path) {
            Ok(_) => (),
            Err(msg) => {
                return Err(InternalError(format!(
                    "Storage path failed to create: {}",
                    msg
                )));
            }
        }
    } else {
        let files_to_read = fs::read_dir(path)
            .expect("Error during reading folder..")
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.path()
                        .file_name()
                        .and_then(|n| n.to_str().map(|s| String::from(s)))
                })
            })
            .collect::<Vec<String>>();
        for file_name in files_to_read {
            let mut content_temp = String::new();
            File::open(Path::new(&format!("{}/{}", path, &file_name)))
                .unwrap()
                .read_to_string(&mut content_temp)
                .unwrap();
            storage
                .data
                .push(deserialize_object::<T>(&content_temp).unwrap());
        }
    }
    // Sort data by date created
    storage.data.sort_by(|a, b| {
        a.get_date_created()
            .partial_cmp(&b.get_date_created())
            .unwrap()
    });
    Ok(storage)
}

// TODO: Documentation please
pub fn storage_id_available<T>(storage: &Storage<T>, id_to_check: &str) -> AppResult<()>
where
    T: StorageObject,
{
    for item in &storage.data {
        if item.get_id() == id_to_check {
            return Err(InternalError(format!(
                "{} is not available, already taken.",
                id_to_check
            )));
        }
    }
    Ok(())
}

/// # Add StorageObject to Storage
///
/// Add StorageObject to Storage and returns NO reference.
pub fn add_to_storage<T>(storage: &mut Storage<T>, mut storage_object: T) -> AppResult<()>
where
    T: StorageObject,
{
    // Check if ID available
    storage_id_available(&storage, &storage_object.get_id())?;
    storage_object.set_path(storage.path).unwrap();
    storage_object.save()?;
    storage.data.push(storage_object);
    Ok(())
}

/// # Add StorageObject to Storage and returns reference to it
pub fn add_to_storage_and_return_ref<T>(
    storage: &mut Storage<T>,
    mut storage_object: T,
) -> AppResult<&mut T>
where
    T: StorageObject,
{
    // Check if ID available
    storage_id_available(&storage, &storage_object.get_id())?;
    // TODO: CHeck with test. Does it really work?
    let id: String;
    {
        id = storage_object.get_id().into();
    }
    storage_object.set_path(storage.path)?;
    storage_object.save()?;
    storage.data.push(storage_object);
    let mut storage_result_index = 0;
    for item in &mut storage.data {
        if item.get_id() == id {
            break;
        }
        storage_result_index += 1;
    }
    match storage.data.get_mut(storage_result_index) {
        Some(data_item) => Ok(data_item),
        None => Err(InternalError(
            "Error while getting reference to the new storage item.".into(),
        )),
    }
}

/// # Serialize object<T> -> Result<String, String>
/// Serialize a given object to String
pub fn serialize_object<T: Serialize>(object: &T) -> AppResult<String> {
    match serde_yaml::to_string(object) {
        Ok(result) => Ok(result),
        Err(_) => Err(InternalError(
            "Error while data object serialisation.".into(),
        )),
    }
}

/// # Deserialize &str into object<T>
/// IMPORTANT: deserializable struct currently cannot have &str field.
//  TODO: Lifetime fix for `&str field type.
pub fn deserialize_object<'a, T: ?Sized>(s: &str) -> AppResult<T>
where
    for<'de> T: Deserialize<'de> + 'a,
{
    match serde_yaml::from_str(s) {
        Ok(t) => Ok(t),
        Err(_) => Err(InternalError(
            "Error while data object deserialization.".into(),
        )),
    }
}

/**
 * Save storage into object!
 * TODO: Doc comments + example code
 */
pub fn save_storage_object<T>(storage_object: &T) -> AppResult<()>
where
    T: StorageObject + Serialize + Sized,
{
    // TODO: Proper error handling please!
    File::create(&format!(
        "{}/{}.yml",
        storage_object.get_path().unwrap(),
        storage_object.get_id(),
    ))
    .unwrap()
    .write_all(serialize_object::<T>(storage_object).unwrap().as_bytes())
    .unwrap();
    Ok(())
}

// TODO: Refact tests to properly handle storage create and clean.
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Demo {
        name: String,
        age: u32,
    }

    static TESTDIR_PATH: &'static str = "../data/example";

    #[test]
    fn test_serialize_object() {
        let d = Demo {
            name: "Lorem Ipsum".to_owned(),
            age: 99,
        };
        assert_eq!(
            serialize_object(&d).unwrap(),
            "---\nname: Lorem Ipsum\nage: 99".to_owned()
        );
    }

    #[test]
    fn test_deserialize_object() {
        let object: Demo = deserialize_object("---\nname: Lorem Ipsum\nage: 99").unwrap();
        assert_eq!(object.name, "Lorem Ipsum".to_owned());
        assert_eq!(object.age, 99);
    }

    #[test]
    fn test_storage() {
        #[derive(Serialize, Deserialize)]
        struct Example {
            id: String,
            path: String,
            name: String,
        }
        impl Example {
            fn new(id: &str, path: &str, name: &str) -> Example {
                Example {
                    id: id.to_owned(),
                    path: path.to_owned(),
                    name: name.to_owned(),
                }
            }
        }
        impl StorageObject for Example {
            fn get_id(&self) -> &str {
                &self.id
            }
            fn save(&self) -> AppResult<()> {
                save_storage_object(self)?;
                Ok(())
            }
            fn reload(&mut self) -> AppResult<()> {
                Ok(())
            }
            fn get_path(&self) -> Option<&str> {
                Some(&self.path)
            }
            fn set_path(&mut self, path: &str) -> AppResult<()> {
                self.path = path.into();
                Ok(())
            }
            fn get_date_created(&self) -> DateTime<Utc> {
                Utc::now()
            }
        }
        let mut storage = load_storage::<Example>("data/123423").unwrap();
        for item in 1..3 {
            storage.data.push(Example::new(
                &format!("{}", item),
                "data/storage",
                &format!("{}", item),
            ));
        }

        let obj1 = Example::new("102", "", "102");
        let obj2 = Example::new("102", "", "103");
        let obj3 = Example::new("102", "", "104");

        storage_id_available(&storage, obj1.get_id()).unwrap();
        storage_id_available(&storage, obj2.get_id()).unwrap();
        storage_id_available(&storage, obj3.get_id()).unwrap();

        add_to_storage(&mut storage, obj1).unwrap();
        add_to_storage(&mut storage, obj2).unwrap();
        add_to_storage(&mut storage, obj3).unwrap();

        let mut item =
            add_to_storage_and_return_ref(&mut storage, Example::new("105", TESTDIR_PATH, "105"))
                .unwrap();
        item.name = "1009".to_owned();

        assert_eq!(storage.data.get(0).unwrap().name, "1");
        assert_eq!(storage.data.get(1).unwrap().name, "2");
        assert_eq!(storage.data.get(2).unwrap().name, "102");
        assert_eq!(storage.data.get(3).unwrap().name, "103");
        assert_eq!(storage.data.get(4).unwrap().name, "104");
        assert_eq!(storage.data.get(5).unwrap().name, "1009");

        assert_eq!(storage.data[5].get_path().unwrap(), "data/123423");
        // Remove storage from FS
        storage.remove();
    }

    // #[test]
    // fn test_load_empty_storage() {
    //     let storage = load_storage::<Demo>("../data/demo").unwrap();
    //     assert_eq!(storage.data.len(), 0);
    //     // Remove storage from FS
    //     storage.remove();
    // }

    #[test]
    fn test_storage_load_save() {
        #[derive(Serialize, Deserialize)]
        struct Example {
            id: String,
            path: String,
            name: String,
        }
        impl Example {
            fn new(id: &str, path: &str, name: &str) -> Example {
                Example {
                    id: id.to_owned(),
                    path: path.to_owned(),
                    name: name.to_owned(),
                }
            }
        }
        impl StorageObject for Example {
            fn get_id(&self) -> &str {
                &self.id
            }
            fn save(&self) -> AppResult<()> {
                save_storage_object(self)?;
                Ok(())
            }
            fn reload(&mut self) -> AppResult<()> {
                Ok(())
            }
            fn get_path(&self) -> Option<&str> {
                Some(&self.path)
            }
            fn set_path(&mut self, path: &str) -> AppResult<()> {
                self.path = path.into();
                Ok(())
            }
            fn get_date_created(&self) -> DateTime<Utc> {
                Utc::now()
            }
        }
        // Lets create new storage
        let mut storage = load_storage::<Example>("data/234234j").unwrap();
        add_to_storage(&mut storage, Example::new("1", "", "Apple")).unwrap();
        add_to_storage(&mut storage, Example::new("2", "", "Banana")).unwrap();
        add_to_storage(&mut storage, Example::new("3", "", "Wohoo")).unwrap();

        for item in &storage.data {
            item.save().unwrap();
        }

        drop(storage);

        // Load demo data from storage
        let storage = load_storage::<Example>("data/234234j").unwrap();
        // Check sum of data
        assert_eq!(storage.data.len(), 3);
        // Check content of data
        for item in &storage.data {
            if item.id == "1" {
                assert_eq!(item.name, "Apple".to_owned());
            }
            if item.id == "2" {
                assert_eq!(item.name, "Banana".to_owned());
            }
            if item.id == "3" {
                assert_eq!(item.name, "Wohoo".to_owned());
            }
        }
        // Remove storage from FS
        storage.remove();
    }
}
