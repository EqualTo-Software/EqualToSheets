#![allow(clippy::unwrap_used)]

use crate::language::get_language;
use crate::locale::get_locale;

use crate::expressions::{
    lexer::{Lexer, LexerMode},
    token::TokenType::*,
    token::{Error, OpSum},
};

fn new_lexer(formula: &str, a1_mode: bool) -> Lexer {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    let mode = if a1_mode {
        LexerMode::A1
    } else {
        LexerMode::R1C1
    };
    Lexer::new(formula, mode, locale, language)
}

#[test]
fn test_number_zero() {
    let mut lx = new_lexer("0", true);
    assert_eq!(lx.next_token(), NUMBER(0.0));
    assert_eq!(lx.next_token(), EOF);
}
#[test]
fn test_number_integer() {
    let mut lx = new_lexer("42", true);
    assert_eq!(lx.next_token(), NUMBER(42.0));
    assert_eq!(lx.next_token(), EOF);
}
#[test]
fn test_number_pi() {
    let mut lx = new_lexer("3.415", true);
    assert_eq!(lx.next_token(), NUMBER(3.415));
    assert_eq!(lx.next_token(), EOF);
}
#[test]
fn test_number_less_than_one() {
    let mut lx = new_lexer(".1415", true);
    assert_eq!(lx.next_token(), NUMBER(0.1415));
    assert_eq!(lx.next_token(), EOF);
}
#[test]
fn test_number_less_than_one_bis() {
    let mut lx = new_lexer("0.1415", true);
    assert_eq!(lx.next_token(), NUMBER(0.1415));
    assert_eq!(lx.next_token(), EOF);
}
#[test]
fn test_number_scientific() {
    let mut lx = new_lexer("1.1415e12", true);
    assert_eq!(lx.next_token(), NUMBER(1.1415e12));
    assert_eq!(lx.next_token(), EOF);
}
#[test]
fn test_number_scientific_1() {
    let mut lx = new_lexer("2.4e-12", true);
    assert_eq!(lx.next_token(), NUMBER(2.4e-12));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_not_a_number() {
    let mut lx = new_lexer("..", true);
    assert!(matches!(lx.next_token(), ILLEGAL(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_string() {
    let mut lx = new_lexer("\"Hello World!\"", true);
    assert_eq!(lx.next_token(), STRING("Hello World!".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_string_unicode() {
    let mut lx = new_lexer("\"你好，世界！\"", true);
    assert_eq!(lx.next_token(), STRING("你好，世界！".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_boolean() {
    let mut lx = new_lexer("FALSE", true);
    assert_eq!(lx.next_token(), BOOLEAN(false));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_boolean_true() {
    let mut lx = new_lexer("True", true);
    assert_eq!(lx.next_token(), BOOLEAN(true));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference() {
    let mut lx = new_lexer("A1", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: None,
            column: 1,
            row: 1,
            absolute_column: false,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_absolute() {
    let mut lx = new_lexer("$A$1", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: None,
            column: 1,
            row: 1,
            absolute_column: true,
            absolute_row: true,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_absolute_1() {
    let mut lx = new_lexer("AB$12", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: None,
            column: 28,
            row: 12,
            absolute_column: false,
            absolute_row: true,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_absolute_2() {
    let mut lx = new_lexer("$CC234", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: None,
            column: 81,
            row: 234,
            absolute_column: true,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_sheet() {
    let mut lx = new_lexer("Sheet1!C34", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: Some("Sheet1".to_string()),
            column: 3,
            row: 34,
            absolute_column: false,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_sheet_unicode() {
    // Not that also tests the '!'
    let mut lx = new_lexer("'A € world!'!C34", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: Some("A € world!".to_string()),
            column: 3,
            row: 34,
            absolute_column: false,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_sheet_unicode_absolute() {
    let mut lx = new_lexer("'A €'!$C$34", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: Some("A €".to_string()),
            column: 3,
            row: 34,
            absolute_column: true,
            absolute_row: true,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_unmatched_quote() {
    let mut lx = new_lexer("'A €!$C$34", true);
    assert!(matches!(lx.next_token(), ILLEGAL(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_sum() {
    let mut lx = new_lexer("2.4+3.415", true);
    assert_eq!(lx.next_token(), NUMBER(2.4));
    assert_eq!(lx.next_token(), SUM(OpSum::Add));
    assert_eq!(lx.next_token(), NUMBER(3.415));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_sum_1() {
    let mut lx = new_lexer("A2 + 'First Sheet'!$B$3", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: None,
            column: 1,
            row: 2,
            absolute_column: false,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), SUM(OpSum::Add));
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: Some("First Sheet".to_string()),
            column: 2,
            row: 3,
            absolute_column: true,
            absolute_row: true,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_value() {
    let mut lx = new_lexer("#VALUE!", true);
    assert_eq!(lx.next_token(), ERROR(Error::VALUE));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_error() {
    let mut lx = new_lexer("#ERROR!", true);
    assert_eq!(lx.next_token(), ERROR(Error::ERROR));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_div() {
    let mut lx = new_lexer("#DIV/0!", true);
    assert_eq!(lx.next_token(), ERROR(Error::DIV));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_na() {
    let mut lx = new_lexer("#N/A", true);
    assert_eq!(lx.next_token(), ERROR(Error::NA));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_name() {
    let mut lx = new_lexer("#NAME?", true);
    assert_eq!(lx.next_token(), ERROR(Error::NAME));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_num() {
    let mut lx = new_lexer("#NUM!", true);
    assert_eq!(lx.next_token(), ERROR(Error::NUM));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_calc() {
    let mut lx = new_lexer("#CALC!", true);
    assert_eq!(lx.next_token(), ERROR(Error::CALC));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_spill() {
    let mut lx = new_lexer("#SPILL!", true);
    assert_eq!(lx.next_token(), ERROR(Error::SPILL));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_circ() {
    let mut lx = new_lexer("#CIRC!", true);
    assert_eq!(lx.next_token(), ERROR(Error::CIRC));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_error_invalid() {
    let mut lx = new_lexer("#VALU!", true);
    assert!(matches!(lx.next_token(), ILLEGAL(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_add_errors() {
    let mut lx = new_lexer("#DIV/0!+#NUM!", true);
    assert_eq!(lx.next_token(), ERROR(Error::DIV));
    assert_eq!(lx.next_token(), SUM(OpSum::Add));
    assert_eq!(lx.next_token(), ERROR(Error::NUM));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_variable_name() {
    let mut lx = new_lexer("MyVar", true);
    assert_eq!(lx.next_token(), IDENT("MyVar".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_last_reference() {
    let mut lx = new_lexer("XFD1048576", true);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: None,
            column: 16384,
            row: 1048576,
            absolute_column: false,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_not_a_reference() {
    let mut lx = new_lexer("XFE10", true);
    assert_eq!(lx.next_token(), IDENT("XFE10".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_r1c1() {
    let mut lx = new_lexer("R1C1", false);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: None,
            column: 1,
            row: 1,
            absolute_column: true,
            absolute_row: true,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_r1c1_true() {
    let mut lx = new_lexer("R1C1", true);
    // NOTE: This is what google docs does.
    // Excel will not let you enter this formula.
    // Online Excel will let you and will mark the cell as in Error
    assert!(matches!(lx.next_token(), ILLEGAL(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_name_r1c1p() {
    let mut lx = new_lexer("R1C1P", false);
    assert_eq!(lx.next_token(), IDENT("R1C1P".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_name_wrong_ref() {
    let mut lx = new_lexer("Sheet1!2", false);
    assert!(matches!(lx.next_token(), ILLEGAL(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_1() {
    let mut lx = new_lexer("Sheet1!R[1]C[2]", false);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: Some("Sheet1".to_string()),
            column: 2,
            row: 1,
            absolute_column: false,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_quotes() {
    let mut lx = new_lexer("'Sheet 1'!R[1]C[2]", false);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: Some("Sheet 1".to_string()),
            column: 2,
            row: 1,
            absolute_column: false,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_escape_quotes() {
    let mut lx = new_lexer("'Sheet ''one'' 1'!R[1]C[2]", false);
    assert_eq!(
        lx.next_token(),
        REFERENCE {
            sheet: Some("Sheet 'one' 1".to_string()),
            column: 2,
            row: 1,
            absolute_column: false,
            absolute_row: false,
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_reference_unfinished_quotes() {
    let mut lx = new_lexer("'Sheet 1!R[1]C[2]", false);
    assert!(matches!(lx.next_token(), ILLEGAL(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_round_function() {
    let mut lx = new_lexer("ROUND", false);
    assert_eq!(lx.next_token(), IDENT("ROUND".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_ident_with_underscore() {
    let mut lx = new_lexer("_IDENT", false);
    assert_eq!(lx.next_token(), IDENT("_IDENT".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_ident_with_period() {
    let mut lx = new_lexer("IDENT.IFIER", false);
    assert_eq!(lx.next_token(), IDENT("IDENT.IFIER".to_string()));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_ident_cannot_start_with_period() {
    let mut lx = new_lexer(".IFIER", false);
    assert!(matches!(lx.next_token(), ILLEGAL(_)));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn test_xlfn() {
    let mut lx = new_lexer("_xlfn.MyVar", true);
    assert_eq!(lx.next_token(), IDENT("MyVar".to_string()));
    assert_eq!(lx.next_token(), EOF);
}
