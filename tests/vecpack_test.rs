use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use storaget::*;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct Car {
    id: usize,
    name: String,
    hp: u32,
}

impl Car {
    pub fn new(id: usize, name: String, hp: u32) -> Self {
        Car { id, name, hp }
    }
}

impl<I> VecPackMember<I> for Car
where
    I: std::fmt::Display,
{
    fn get_id(&self) -> usize {
        self.0
    }
}

#[test]
fn test_load_or_init() {
    let meaning_of_life: PackResult<VecPack<Car>> =
        VecPack::load_or_init(PathBuf::from("data/vecpack_test"));
    assert_eq!(meaning_of_life.is_ok(), true);
    assert_eq!(meaning_of_life.unwrap().len(), 0);
}
