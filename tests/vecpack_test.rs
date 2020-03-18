use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use storaget::*;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct Car {
    pub id: usize,
    pub name: String,
    pub hp: u32,
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

fn create_dummy_vecpack(path: PathBuf) -> VecPack<Car> {
    let mut meaning_of_life: VecPack<Car> =
        VecPack::load_or_init(path).unwrap();
    meaning_of_life
        .insert(Car::new(1, "CarSmall".to_string(), 150))
        .unwrap();
    meaning_of_life
        .insert(Car::new(2, "CarBig".to_string(), 650))
        .unwrap();
    meaning_of_life
        .insert(Car::new(3, "CarMedium".to_string(), 250))
        .unwrap();
    meaning_of_life
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

#[test]
fn test_vecpack_as_mut() {
    let mut cars =
        create_dummy_vecpack(PathBuf::from("data/vecpack_test_as_mut"));
    cars.into_iter().for_each(|i| i.as_mut().hp = 1);
    drop(cars);
    let cars = create_dummy_vecpack(PathBuf::from("data/vecpack_test_as_mut"));
    assert_eq!(cars.get(0).unwrap().hp, 1);
}

#[test]
fn test_vecpack_find_id() {
    let cars = create_dummy_vecpack(PathBuf::from("data/vecpack_test_find_id"));
    drop(cars);
    let cars = create_dummy_vecpack(PathBuf::from("data/vecpack_test_find_id"));
    assert_eq!(cars.find_id(3).is_ok(), true);
    assert_eq!(cars.find_id(1).unwrap().hp, 150);
    assert_eq!(cars.find_id(2).unwrap().hp, 650);
    assert_eq!(cars.find_id(3).unwrap().hp, 250);
}

#[test]
fn test_vecpack_find_id_mut() {
    let mut cars =
        create_dummy_vecpack(PathBuf::from("data/vecpack_test_find_id_mut"));
    cars.find_id_mut(1).unwrap().update(|i| i.hp = 1).unwrap();
    cars.find_id_mut(2).unwrap().update(|i| i.hp = 11).unwrap();
    cars.find_id_mut(3).unwrap().update(|i| i.hp = 111).unwrap();

    drop(cars);
    let cars =
        create_dummy_vecpack(PathBuf::from("data/vecpack_test_find_id_mut"));

    assert_eq!(cars.find_id(1).unwrap().hp, 1);
    assert_eq!(cars.find_id(2).unwrap().hp, 11);
    assert_eq!(cars.find_id(3).unwrap().hp, 111);
}
