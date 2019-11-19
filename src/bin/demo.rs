extern crate storaget;

use rand::prelude::*;
use std::sync::{Arc, Mutex};
use storaget::{Storage, *};
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
// Generate random string with a given lenght.
// Using english alphabet + integers + a few other basic characters.
fn build_string(length: i32) -> String {
    let mut string = "".to_owned();
    let chars = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        ' ', '_', '-', '.', '*',
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
// Storage builder for tests.
fn build_user_storage_of_dummies(size_of_storage: i32) -> Storage<User> {
    let mut storage: Storage<User> = Storage::new("data");
    for index in 0..size_of_storage {
        let mut user = User::new(
            &format!("{}", index),
            &build_string(20),
            rand::thread_rng().gen_range(10, 70),
        );
        user.description = build_string(1000);
        user.favorit_colors = vec![build_string(5), build_string(10), build_string(7)];
        user.favorit_numbers = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        storage.add_to_storage(user).unwrap();
    }
    storage
}

fn into_background_process(st: storaget::DataObject<User>) {
    std::thread::spawn(move || {
        std::thread::sleep_ms(3000);
        st.update(|u| {
            u.set_name("Yay!");
            u.age = 0;
        });
        std::thread::sleep_ms(3000);
        st.update(|u| {
            u.set_name("Peti again!");
            u.age = 31;
        });
    });
}

fn main() {
    // let storage = build_user_storage_of_dummies(100000);
    // for index in 1..1000 {
    //     std::thread::sleep_ms(300);
    //     println!("Hi {}", index);
    // }
    let mut storage: Storage<User> = Storage::new("data");
    storage
        .add_to_storage(User::new("1", "Kriszti", 27))
        .unwrap();
    storage.add_to_storage(User::new("2", "Peti", 31)).unwrap();
    storage.add_to_storage(User::new("3", "Gabi", 27)).unwrap();
    storage.add_to_storage(User::new("4", "Rozi", 0)).unwrap();

    let holder: Arc<Mutex<Storage<User>>> = Arc::new(Mutex::from(storage));

    let st = holder.lock().unwrap().get_by_id("2").unwrap();
    into_background_process(st);

    loop {
        let user = holder.lock().unwrap().get_by_id("2").unwrap();
        let (name, age) = user.get(|u| (u.get_name().to_owned(), u.age));
        println!("User name is: {}; age is: {}", name, age);
        std::thread::sleep_ms(300);
    }
}
