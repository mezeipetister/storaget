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

use crate::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

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
///
/// ```rust
/// use core_lib::storage::*;
/// use serde::{Deserialize, Serialize};
/// #[derive(Serialize, Deserialize)]
/// struct Animal {
///     id: u32,
///     name: String,
/// }
/// let storage = load_storage::<Animal>("../data/animals").unwrap();
/// storage.remove();
/// assert_eq!(storage.data.len(), 0);
/// ```
pub fn load_storage<'a, T>(path: &'static str) -> StorageResult<Storage<T>>
where
    for<'de> T: Deserialize<'de> + 'a + StorageObject + Serialize,
{
    manage_path(path)?;
    let storage: Storage<T> = Storage::new(path);
    for item in load::<T>(path)? {
        storage.add_to_storage(item)?;
    }
    Ok(storage)
}

pub(crate) fn manage_path(path: &'static str) -> StorageResult<()> {
    if !Path::new(path).exists() {
        match fs::create_dir_all(path) {
            Ok(_) => return Ok(()),
            Err(err) => {
                return Err(Error::InternalError(format!("{}", err)));
            }
        }
    }
    Ok(())
}

pub(crate) fn remove_path(path: &'static str) -> StorageResult<()> {
    match fs::remove_dir_all(path) {
        Ok(_) => return Ok(()),
        Err(err) => {
            return Err(Error::InternalError(format!("{}", err)));
        }
    }
}

fn load<'a, T>(path: &'static str) -> StorageResult<Vec<T>>
where
    for<'de> T: Deserialize<'de> + 'a,
{
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
    let mut st_temp = Vec::new();
    for file_name in files_to_read {
        let mut content_temp = String::new();
        File::open(Path::new(&format!("{}/{}", path, &file_name)))
            .unwrap()
            .read_to_string(&mut content_temp)
            .unwrap();
        st_temp.push(deserialize_object::<T>(&content_temp).unwrap());
    }
    return Ok(st_temp);
}

/// # Serialize object<T> -> Result<String, String>
/// Serialize a given object to String
/// ```rust
/// use serde::{Deserialize, Serialize};
/// use core_lib::storage::*;
/// #[derive(Serialize, Deserialize)]
/// struct Animal {
///     id: u32,
///     name: String,
/// }
/// let dog = Animal { id: 1, name: "Puppy Joe".to_owned() };
/// let serialized_object = serialize_object(&dog).unwrap();
/// assert_eq!(serialized_object, "---\nid: 1\nname: Puppy Joe".to_owned());
/// ```
pub fn serialize_object<T>(object: &T) -> StorageResult<String>
where
    T: Serialize,
{
    match serde_yaml::to_string(object) {
        Ok(result) => Ok(result),
        Err(err) => Err(Error::SerializeError(format!("{}", err))),
    }
}

/// # Deserialize &str into object<T>
/// ```rust
/// use serde::{Deserialize, Serialize};
/// use core_lib::storage::*;
/// #[derive(Serialize, Deserialize)]
/// struct Animal {
///     id: u32,
///     name: String,
/// }
/// let animal: Animal = deserialize_object("---\nid: 1\nname: Puppy Joe").unwrap();
/// assert_eq!(animal.id, 1);
/// assert_eq!(animal.name, "Puppy Joe".to_owned());
/// ```
/// IMPORTANT: deserializable struct currently cannot have &str field.
//  TODO: Lifetime fix for `&str field type.
pub fn deserialize_object<'a, T: ?Sized>(s: &str) -> StorageResult<T>
where
    for<'de> T: Deserialize<'de> + 'a,
{
    match serde_yaml::from_str(s) {
        Ok(t) => Ok(t),
        Err(err) => Err(Error::DeserializeError(format!("{}", err))),
    }
}

/**
 * Save storage into object!
 * TODO: Doc comments + example code
 */
pub fn save_storage_object<'a, T>(storage_object: &'a T, path: &'static str) -> StorageResult<()>
where
    T: StorageObject + Serialize,
{
    // TODO: Proper error handling please!
    File::create(&format!("{}/{}.yml", path, storage_object.get_id(),))?
        .write(serialize_object::<T>(storage_object)?.as_bytes())?;
    Ok(())
}
