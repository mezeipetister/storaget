use std::path::PathBuf;
use storaget::*;

#[test]
fn test_load_or_init() {
    let meaning_of_life: PackResult<Pack<i32>> =
        Pack::load_or_init(PathBuf::from("data/pack_test"), "meaning_of_life");
    assert_eq!(meaning_of_life.is_ok(), true);
}

#[test]
fn test_update() {
    let mut meaning_of_life: Pack<i32> = Pack::load_or_init(
        PathBuf::from("data/pack_test"),
        "meaning_of_life_update",
    )
    .unwrap();
    *(meaning_of_life.as_mut()) = 17;
    &mut meaning_of_life.update(|i| *i = 42);
    assert_eq!(*(meaning_of_life), 42);
}

#[test]
fn test_as_mut() {
    let mut meaning_of_life: Pack<i32> = Pack::load_or_init(
        PathBuf::from("data/pack_test"),
        "meaning_of_life_as_mut",
    )
    .unwrap();
    *(meaning_of_life.as_mut()) = 17;
    assert_eq!(*(meaning_of_life), 17);
    *(meaning_of_life.as_mut()) = 42;
    assert_eq!(*(meaning_of_life), 42);
}

#[test]
fn test_as_deref() {
    let mut meaning_of_life: Pack<i32> = Pack::load_or_init(
        PathBuf::from("data/pack_test"),
        "meaning_of_life_deref",
    )
    .unwrap();
    *(meaning_of_life.as_mut()) = 42;
    // Manually drop
    // meaning_of_life variable
    drop(meaning_of_life);
    // Init it again
    // and read the stored value
    let meaning_of_life: Pack<i32> = Pack::load_or_init(
        PathBuf::from("data/pack_test"),
        "meaning_of_life_deref",
    )
    .unwrap();
    assert_eq!(*(meaning_of_life), 42);
}
