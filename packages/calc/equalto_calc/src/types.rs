use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

use crate::expressions::token::Error;

// Useful for `#[serde(default = "default_as_true")]`
fn default_as_true() -> bool {
    true
}

fn default_as_false() -> bool {
    false
}

// Useful for `#[serde(skip_serializing_if = "is_true")]`
fn is_true(b: &bool) -> bool {
    *b
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_zero(num: &i32) -> bool {
    *num == 0
}

fn is_default_alignment(o: &Option<Alignment>) -> bool {
    o.is_none() || *o == Some(Alignment::default())
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Metadata {
    pub application: String,
    pub app_version: String,
    pub creator: String,
    pub last_modified_by: String,
    pub created: String,       // "2020-08-06T21:20:53Z",
    pub last_modified: String, //"2020-11-20T16:24:35"
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct WorkbookSettings {
    pub tz: String,
    pub locale: String,
}
/// An internal representation of an EqualTo Workbook
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Workbook {
    pub shared_strings: Vec<String>,
    pub defined_names: Vec<DefinedName>,
    pub worksheets: Vec<Worksheet>,
    pub styles: Styles,
    pub name: String,
    pub settings: WorkbookSettings,
    pub metadata: Metadata,
}

/// A defined name. The `sheet_id` is the sheet index in case the name is local
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DefinedName {
    pub name: String,
    pub formula: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sheet_id: Option<u32>,
}

// TODO: Move to worksheet.rs make frozen_rows/columns private and u32
/// Internal representation of a worksheet Excel object

/// * state:
///    18.18.68 ST_SheetState (Sheet Visibility Types)
///    hidden, veryHidden, visible
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SheetState {
    Visible,
    Hidden,
    VeryHidden,
}

impl Display for SheetState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SheetState::Visible => write!(formatter, "visible"),
            SheetState::Hidden => write!(formatter, "hidden"),
            SheetState::VeryHidden => write!(formatter, "veryHidden"),
        }
    }
}

/// Internal representation of a worksheet Excel object
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Worksheet {
    pub dimension: String,
    pub cols: Vec<Col>,
    pub rows: Vec<Row>,
    pub name: String,
    pub sheet_data: SheetData,
    pub shared_formulas: Vec<String>,
    pub sheet_id: u32,
    pub state: SheetState,
    #[serde(default = "Color::new")]
    #[serde(skip_serializing_if = "Color::is_none")]
    pub color: Color,
    pub merge_cells: Vec<String>,
    pub comments: Vec<Comment>,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub frozen_rows: i32,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_zero")]
    pub frozen_columns: i32,
}

/// Internal representation of Excel's sheet_data
/// It is row first and because of this all of our API's should be row first
pub type SheetData = HashMap<i32, HashMap<i32, Cell>>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Row {
    pub height: f64,
    pub r: i32,
    pub custom_format: bool,
    pub custom_height: bool,
    pub s: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Col {
    pub min: i32,
    pub max: i32,
    pub width: f64,
    pub custom_width: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Color {
    None,
    RGB(String),
}

impl Color {
    pub fn is_none(&self) -> bool {
        matches!(self, Color::None)
    }
    pub fn new() -> Color {
        Color::None
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new()
    }
}

/// Cell type enum matching Excel TYPE() function values.
#[derive(Debug, Eq, PartialEq)]
pub enum CellType {
    Number = 1,
    Text = 2,
    LogicalValue = 4,
    ErrorValue = 16,
    Array = 64,
    CompoundData = 128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "t", deny_unknown_fields)]
pub enum Cell {
    #[serde(rename = "empty")]
    EmptyCell { s: i32 },
    #[serde(rename = "b")]
    BooleanCell { v: bool, s: i32 },
    #[serde(rename = "n")]
    NumberCell { v: f64, s: i32 },
    // Maybe we should not have this type. In Excel this is just a string
    #[serde(rename = "e")]
    ErrorCell { ei: Error, s: i32 },
    // Always a shared string
    #[serde(rename = "s")]
    SharedString { si: i32, s: i32 },
    // Non evaluated Formula
    #[serde(rename = "u")]
    CellFormula { f: i32, s: i32 },
    #[serde(rename = "fb")]
    CellFormulaBoolean { f: i32, v: bool, s: i32 },
    #[serde(rename = "fn")]
    CellFormulaNumber { f: i32, v: f64, s: i32 },
    // always inline string
    #[serde(rename = "str")]
    CellFormulaString { f: i32, v: String, s: i32 },
    #[serde(rename = "fe")]
    CellFormulaError {
        f: i32,
        ei: Error,
        s: i32,
        // Origin: Sheet3!C4
        o: String,
        // Error Message: "Not implemented function"
        m: String,
    },
    // TODO: Array formulas
}

impl Default for Cell {
    fn default() -> Self {
        Cell::EmptyCell { s: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Comment {
    pub text: String,
    pub author_name: String,
    pub author_id: Option<String>,
    pub cell_ref: String,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Styles {
    pub num_fmts: Vec<NumFmt>,
    pub fonts: Vec<Font>,
    pub fills: Vec<Fill>,
    pub borders: Vec<Border>,
    pub cell_style_xfs: Vec<CellStyleXfs>,
    pub cell_xfs: Vec<CellXfs>,
    pub cell_styles: Vec<CellStyles>,
}

impl Default for Styles {
    fn default() -> Self {
        Styles {
            num_fmts: vec![],
            fonts: vec![Default::default()],
            fills: vec![Default::default()],
            borders: vec![Default::default()],
            cell_style_xfs: vec![Default::default()],
            cell_xfs: vec![Default::default()],
            cell_styles: vec![Default::default()],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct NumFmt {
    pub num_fmt_id: i32,
    pub format_code: String,
}

impl Default for NumFmt {
    fn default() -> Self {
        NumFmt {
            num_fmt_id: 0,
            format_code: "general".to_string(),
        }
    }
}

// ST_FontScheme simple type (§18.18.33).
// Usually major fonts are used for styles like headings,
// and minor fonts are used for body and paragraph text.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FontScheme {
    Minor,
    Major,
    None,
}

impl Default for FontScheme {
    fn default() -> Self {
        FontScheme::Minor
    }
}

impl Display for FontScheme {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FontScheme::Minor => write!(formatter, "minor"),
            FontScheme::Major => write!(formatter, "major"),
            FontScheme::None => write!(formatter, "none"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Font {
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub strike: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub u: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub b: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub i: bool,
    pub sz: i32,
    pub color: Color,
    pub name: String,
    // This is the font family fallback
    // 1 -> serif
    // 2 -> sans serif
    // 3 -> monospaced
    // ...
    pub family: i32,
    pub scheme: FontScheme,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            strike: false,
            u: false,
            b: false,
            i: false,
            sz: 11,
            color: Color::RGB("#000000".to_string()),
            name: "Calibri".to_string(),
            family: 2,
            scheme: FontScheme::Minor,
        }
    }
}

// TODO: Maybe use an enum for the pattern_type values here?
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Fill {
    pub pattern_type: String,
    #[serde(default = "Color::new")]
    #[serde(skip_serializing_if = "Color::is_none")]
    pub fg_color: Color,
    #[serde(default = "Color::new")]
    #[serde(skip_serializing_if = "Color::is_none")]
    pub bg_color: Color,
}

impl Default for Fill {
    fn default() -> Self {
        Fill {
            pattern_type: "none".to_string(),
            fg_color: Default::default(),
            bg_color: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum HorizontalAlignment {
    Center,
    CenterContinuous,
    Distributed,
    Fill,
    General,
    Justify,
    Left,
    Right,
}

// Note that alignment in "General" depends on type
impl Default for HorizontalAlignment {
    fn default() -> Self {
        Self::General
    }
}

impl HorizontalAlignment {
    fn is_default(&self) -> bool {
        self == &HorizontalAlignment::default()
    }
}

// FIXME: Is there a way to generate this automatically?
impl Display for HorizontalAlignment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HorizontalAlignment::Center => write!(formatter, "center"),
            HorizontalAlignment::CenterContinuous => write!(formatter, "centerContinuous"),
            HorizontalAlignment::Distributed => write!(formatter, "distributed"),
            HorizontalAlignment::Fill => write!(formatter, "fill"),
            HorizontalAlignment::General => write!(formatter, "general"),
            HorizontalAlignment::Justify => write!(formatter, "justify"),
            HorizontalAlignment::Left => write!(formatter, "left"),
            HorizontalAlignment::Right => write!(formatter, "right"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum VerticalAlignment {
    Bottom,
    Center,
    Distributed,
    Justify,
    Top,
}

impl VerticalAlignment {
    fn is_default(&self) -> bool {
        self == &VerticalAlignment::default()
    }
}

impl Default for VerticalAlignment {
    fn default() -> Self {
        Self::Bottom
    }
}

impl Display for VerticalAlignment {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VerticalAlignment::Bottom => write!(formatter, "bottom"),
            VerticalAlignment::Center => write!(formatter, "center"),
            VerticalAlignment::Distributed => write!(formatter, "distributed"),
            VerticalAlignment::Justify => write!(formatter, "justify"),
            VerticalAlignment::Top => write!(formatter, "top"),
        }
    }
}

// 1762
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct Alignment {
    #[serde(default)]
    #[serde(skip_serializing_if = "HorizontalAlignment::is_default")]
    pub horizontal: HorizontalAlignment,
    #[serde(skip_serializing_if = "VerticalAlignment::is_default")]
    #[serde(default)]
    pub vertical: VerticalAlignment,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub wrap_text: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CellStyleXfs {
    pub num_fmt_id: i32,
    pub font_id: i32,
    pub fill_id: i32,
    pub border_id: i32,
    #[serde(default = "default_as_true")]
    #[serde(skip_serializing_if = "is_true")]
    pub apply_number_format: bool,
    #[serde(default = "default_as_true")]
    #[serde(skip_serializing_if = "is_true")]
    pub apply_border: bool,
    #[serde(default = "default_as_true")]
    #[serde(skip_serializing_if = "is_true")]
    pub apply_alignment: bool,
    #[serde(default = "default_as_true")]
    #[serde(skip_serializing_if = "is_true")]
    pub apply_protection: bool,
    #[serde(default = "default_as_true")]
    #[serde(skip_serializing_if = "is_true")]
    pub apply_font: bool,
    #[serde(default = "default_as_true")]
    #[serde(skip_serializing_if = "is_true")]
    pub apply_fill: bool,
}

impl Default for CellStyleXfs {
    fn default() -> Self {
        CellStyleXfs {
            num_fmt_id: 0,
            font_id: 0,
            fill_id: 0,
            border_id: 0,
            apply_number_format: true,
            apply_border: true,
            apply_alignment: true,
            apply_protection: true,
            apply_font: true,
            apply_fill: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct CellXfs {
    pub xf_id: i32,
    pub num_fmt_id: i32,
    pub font_id: i32,
    pub fill_id: i32,
    pub border_id: i32,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub apply_number_format: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub apply_border: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub apply_alignment: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub apply_protection: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub apply_font: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub apply_fill: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub quote_prefix: bool,
    #[serde(skip_serializing_if = "is_default_alignment")]
    pub alignment: Option<Alignment>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CellStyles {
    pub name: String,
    pub xf_id: i32,
    pub builtin_id: i32,
}

impl Default for CellStyles {
    fn default() -> Self {
        CellStyles {
            name: "normal".to_string(),
            xf_id: 0,
            builtin_id: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    Thin,
    Medium,
    Thick,
    Double,
    Dotted,
    SlantDashDot,
    MediumDashed,
    MediumDashDotDot,
    MediumDashDot,
}

impl Display for BorderStyle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BorderStyle::Thin => write!(formatter, "thin"),
            BorderStyle::Thick => write!(formatter, "thick"),
            BorderStyle::SlantDashDot => write!(formatter, "slantdashdot"),
            BorderStyle::MediumDashed => write!(formatter, "mediumdashed"),
            BorderStyle::MediumDashDotDot => write!(formatter, "mediumdashdotdot"),
            BorderStyle::MediumDashDot => write!(formatter, "mediumdashdot"),
            BorderStyle::Medium => write!(formatter, "medium"),
            BorderStyle::Double => write!(formatter, "double"),
            BorderStyle::Dotted => write!(formatter, "dotted"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct BorderItem {
    pub style: BorderStyle,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct Border {
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub diagonal_up: bool,
    #[serde(default = "default_as_false")]
    #[serde(skip_serializing_if = "is_false")]
    pub diagonal_down: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<BorderItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagonal: Option<BorderItem>,
}
