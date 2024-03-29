#![allow(clippy::unwrap_used)]

use std::env;

use crate::model::Model;
use crate::test::util::new_empty_model;

// FIXME: Maybe use some sort of decorator for this?
// I would be easy to forget to remove the env variable
// 14:44 20 Mar 2023 Berlin
const TIMESTAMP_2023: i64 = 1679319865208;

#[test]
fn today_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=TODAY()");
    model._set("A2", "=TEXT(A1, \"yyyy/m/d\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"08/11/2022");
    assert_eq!(model._get_text("A2"), *"2022/11/8");
}

#[test]
fn today_with_wrong_tz() {
    let model = Model::new_empty("model", "en", "Wrong Timezone");
    assert!(model.is_err());
}

#[test]
fn now_basic_utc() {
    env::set_var("TIMESTAMP", format!("{TIMESTAMP_2023}"));
    let mut model = Model::new_empty("model", "en", "UTC").unwrap();
    model._set("A1", "=TODAY()");
    model._set("A2", "=NOW()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"20/03/2023");
    assert_eq!(model._get_text("A2"), *"45005.572511574");
    env::remove_var("TIMESTAMP");
}

#[test]
fn now_basic_europe_berlin() {
    env::set_var("TIMESTAMP", format!("{TIMESTAMP_2023}"));
    let mut model = Model::new_empty("model", "en", "Europe/Berlin").unwrap();
    model._set("A1", "=TODAY()");
    model._set("A2", "=NOW()");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"20/03/2023");
    // This is UTC + 1 hour: 45005.572511574 + 1/24
    assert_eq!(model._get_text("A2"), *"45005.614178241");
    env::remove_var("TIMESTAMP");
}
