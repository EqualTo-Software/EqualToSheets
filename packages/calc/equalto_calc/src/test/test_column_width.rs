#![allow(clippy::unwrap_used)]

use crate::constants::{COLUMN_WIDTH_FACTOR, DEFAULT_COLUMN_WIDTH};
use crate::test::util::new_empty_model;
use crate::types::Col;

#[test]
fn test_column_width() {
    let mut model = new_empty_model();
    let cols = vec![Col {
        custom_width: false,
        max: 16384,
        min: 1,
        style: Some(6),
        width: 8.7,
    }];
    model.workbook.worksheets[0].cols = cols;
    model.set_column_width(0, 2, 30.0);
    assert_eq!(model.workbook.worksheets[0].cols.len(), 3);
    assert!((model.get_column_width(0, 1) - DEFAULT_COLUMN_WIDTH).abs() < f64::EPSILON);
    assert!((model.get_column_width(0, 2) - 30.0).abs() < f64::EPSILON);
    assert!((model.get_column_width(0, 3) - DEFAULT_COLUMN_WIDTH).abs() < f64::EPSILON);
    assert_eq!(model.get_cell_style_index(0, 23, 2), 6);
}

#[test]
fn test_column_width_lower_edge() {
    let mut model = new_empty_model();
    let cols = vec![Col {
        custom_width: true,
        max: 16,
        min: 5,
        style: Some(1),
        width: 10.0,
    }];
    model.workbook.worksheets[0].cols = cols;
    model.set_column_width(0, 5, 30.0);
    assert_eq!(model.workbook.worksheets[0].cols.len(), 2);
    assert!((model.get_column_width(0, 4) - DEFAULT_COLUMN_WIDTH).abs() < f64::EPSILON);
    assert!((model.get_column_width(0, 5) - 30.0).abs() < f64::EPSILON);
    assert!((model.get_column_width(0, 6) - 10.0 * COLUMN_WIDTH_FACTOR).abs() < f64::EPSILON);
    assert_eq!(model.get_cell_style_index(0, 23, 5), 1);
}

#[test]
fn test_column_width_higher_edge() {
    let mut model = new_empty_model();
    let cols = vec![Col {
        custom_width: true,
        max: 16,
        min: 5,
        style: Some(1),
        width: 10.0,
    }];
    model.workbook.worksheets[0].cols = cols;
    model.set_column_width(0, 16, 30.0);
    assert_eq!(model.workbook.worksheets[0].cols.len(), 2);
    assert!((model.get_column_width(0, 15) - 10.0 * COLUMN_WIDTH_FACTOR).abs() < f64::EPSILON);
    assert!((model.get_column_width(0, 16) - 30.0).abs() < f64::EPSILON);
    assert!((model.get_column_width(0, 17) - DEFAULT_COLUMN_WIDTH).abs() < f64::EPSILON);
    assert_eq!(model.get_cell_style_index(0, 23, 16), 1);
}
