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

impl VecPackMember for Car {
    type Target = usize;
    fn get_id(&self) -> Self::Target {
        self.id
    }
}

#[test]
fn test_vecpack_load_or_init() {
    let meaning_of_life: PackResult<VecPack<Car>> =
        VecPack::load_or_init(PathBuf::from("data/vecpack_test_load_or_init"));
    assert_eq!(meaning_of_life.is_ok(), true);
    assert_eq!((*meaning_of_life.unwrap()).len(), 0);
}

#[test]
fn test_vecpack_insert() {
    let mut meaning_of_life: VecPack<Car> =
        VecPack::load_or_init(PathBuf::from("data/vecpack_test_insert"))
            .unwrap();
    meaning_of_life
        .insert(Car::new(1, "CarSmall".to_string(), 150))
        .unwrap();
    meaning_of_life
        .insert(Car::new(2, "CarBig".to_string(), 650))
        .unwrap();
    meaning_of_life
        .insert(Car::new(3, "CarMedium".to_string(), 250))
        .unwrap();

    assert_eq!((*meaning_of_life).len(), 3);
}
