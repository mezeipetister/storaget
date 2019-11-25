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
extern crate storaget;
use serde::{Deserialize, Serialize};
use storaget::*;

#[derive(Serialize, Deserialize)]
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
impl<'de> StorageMember<'de> for User {
    fn get_id(&self) -> &str {
        &self.id
    }
}

fn main() -> StorageResult<()> {
    let mut users: Storage<User> = Storage::load("data")?;
    users.add_to_storage(User::new("1", "Demo", 11));
    users.add_to_storage(User::new("2", "Demo2", 12));
    Ok(())
}
