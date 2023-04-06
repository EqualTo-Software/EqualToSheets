use core::fmt;

use crate::{
    calc_result::{CalcResult, CellReference},
    expressions::{parser::Node, token::Error},
    model::Model,
};

pub(crate) mod binary_search;
mod date_and_time;
mod engineering;
mod financial;
mod financial_util;
mod information;
mod logical;
mod lookup_and_reference;
mod mathematical;
mod statistical;
mod subtotal;
mod text;
mod text_util;
pub(crate) mod util;
mod xlookup;

/// List of all implemented functions
#[derive(PartialEq, Clone, Debug)]
pub enum Function {
    // Logical
    And,
    False,
    If,
    Iferror,
    Ifna,
    Ifs,
    Not,
    Or,
    Switch,
    True,
    Xor,

    // Mathematical and trigonometry
    Abs,
    Acos,
    Acosh,
    Asin,
    Asinh,
    Atan,
    Atan2,
    Atanh,
    Choose,
    Column,
    Columns,
    Cos,
    Cosh,
    Max,
    Min,
    Pi,
    Power,
    Product,
    Rand,
    Randbetween,
    Round,
    Rounddown,
    Roundup,
    Sin,
    Sinh,
    Sqrt,
    Sqrtpi,
    Sum,
    Sumif,
    Sumifs,
    Tan,
    Tanh,

    // Information
    ErrorType,
    Isblank,
    Iserr,
    Iserror,
    Iseven,
    Isformula,
    Islogical,
    Isna,
    Isnontext,
    Isnumber,
    Isodd,
    Isref,
    Istext,
    Na,
    Sheet,
    Type,

    // Lookup and reference
    Hlookup,
    Index,
    Indirect,
    Lookup,
    Match,
    Offset,
    Row,
    Rows,
    Vlookup,
    Xlookup,

    // Text
    Concat,
    Concatenate,
    Exact,
    Find,
    Left,
    Len,
    Lower,
    Mid,
    Rept,
    Right,
    Search,
    Substitute,
    T,
    Text,
    Textafter,
    Textbefore,
    Textjoin,
    Trim,
    Upper,
    Value,
    Valuetotext,

    // Statistical
    Average,
    Averagea,
    Averageif,
    Averageifs,
    Count,
    Counta,
    Countblank,
    Countif,
    Countifs,
    Maxifs,
    Minifs,

    // Date and time
    Date,
    Day,
    Edate,
    Month,
    Now,
    Today,
    Year,

    // Financial
    Cumipmt,
    Cumprinc,
    Db,
    Ddb,
    Dollarde,
    Dollarfr,
    Effect,
    Fv,
    Ipmt,
    Irr,
    Ispmt,
    Mirr,
    Nominal,
    Nper,
    Npv,
    Pduration,
    Pmt,
    Ppmt,
    Pv,
    Rate,
    Rri,
    Sln,
    Syd,
    Tbilleq,
    Tbillprice,
    Tbillyield,
    Xirr,
    Xnpv,

    // Engineering: Bessel and transcendental functions
    Besseli,
    Besselj,
    Besselk,
    Bessely,
    Erf,
    Erfc,
    ErfcPrecise,
    ErfPrecise,

    // Engineering: Number systems
    Bin2dec,
    Bin2hex,
    Bin2oct,
    Dec2Bin,
    Dec2hex,
    Dec2oct,
    Hex2bin,
    Hex2dec,
    Hex2oct,
    Oct2bin,
    Oct2dec,
    Oct2hex,

    // Engineering: Bit functions
    Bitand,
    Bitlshift,
    Bitor,
    Bitrshift,
    Bitxor,

    // Engineering: Complex functions
    Complex,
    Imabs,
    Imaginary,
    Imargument,
    Imconjugate,
    Imcos,
    Imcosh,
    Imcot,
    Imcsc,
    Imcsch,
    Imdiv,
    Imexp,
    Imln,
    Imlog10,
    Imlog2,
    Impower,
    Improduct,
    Imreal,
    Imsec,
    Imsech,
    Imsin,
    Imsinh,
    Imsqrt,
    Imsub,
    Imsum,
    Imtan,

    // Engineering: Misc function
    Convert,
    Delta,
    Gestep,
    Subtotal,
}

impl Function {
    /// Some functions in Excel like CONCAT are stringified as `_xlfn.CONCAT`.
    pub fn to_xlsx_string(&self) -> String {
        match self {
            Function::Concat => "_xlfn.CONCAT".to_string(),
            Function::Ifna => "_xlfn.IFNA".to_string(),
            Function::Ifs => "_xlfn.IFS".to_string(),
            Function::Maxifs => "_xlfn.MAXIFS".to_string(),
            Function::Minifs => "_xlfn.MINIFS".to_string(),
            Function::Switch => "_xlfn.SWITCH".to_string(),
            Function::Xlookup => "_xlfn.XLOOKUP".to_string(),
            Function::Xor => "_xlfn.XOR".to_string(),
            Function::Textbefore => "_xlfn.TEXTBEFORE".to_string(),
            Function::Textafter => "_xlfn.TEXTAFTER".to_string(),
            Function::Textjoin => "_xlfn.TEXTJOIN".to_string(),
            Function::Rri => "_xlfn.RRI".to_string(),
            Function::Pduration => "_xlfn.PDURATION".to_string(),
            Function::Bitand => "_xlfn.BITAND".to_string(),
            Function::Bitor => "_xlfn.BITOR".to_string(),
            Function::Bitxor => "_xlfn.BITXOR".to_string(),
            Function::Bitlshift => "_xlfn.BITLSHIFT".to_string(),
            Function::Bitrshift => "_xlfn.BITRSHIFT".to_string(),
            Function::Imtan => "_xlfn.IMTAN".to_string(),
            Function::Imsinh => "_xlfn.IMSINH".to_string(),
            Function::Imcosh => "_xlfn.IMCOSH".to_string(),
            Function::Imcot => "_xlfn.IMCOT".to_string(),
            Function::Imcsc => "_xlfn.IMCSC".to_string(),
            Function::Imcsch => "_xlfn.IMCSCH".to_string(),
            Function::Imsec => "_xlfn.IMSEC".to_string(),
            Function::ErfcPrecise => "_xlfn.ERFC.PRECISE".to_string(),
            Function::ErfPrecise => "_xlfn.ERF.PRECISE".to_string(),
            Function::Valuetotext => "_xlfn.VALUETOTEXT".to_string(),
            Function::Isformula => "_xlfn.ISFORMULA".to_string(),
            Function::Sheet => "_xlfn.SHEET".to_string(),
            _ => self.to_string(),
        }
    }

    pub(crate) fn returns_reference(&self) -> bool {
        matches!(self, Function::Indirect | Function::Offset)
    }
    /// Gets the function from the name.
    /// Note that in Excel some (modern) functions are prefixed by `_xlfn.`
    pub fn get_function(name: &str) -> Option<Function> {
        match name.to_ascii_uppercase().as_str() {
            "AND" => Some(Function::And),
            "FALSE" => Some(Function::False),
            "IF" => Some(Function::If),
            "IFERROR" => Some(Function::Iferror),
            "IFNA" | "_XLFN.IFNA" => Some(Function::Ifna),
            "IFS" | "_XLFN.IFS" => Some(Function::Ifs),
            "NOT" => Some(Function::Not),
            "OR" => Some(Function::Or),
            "SWITCH" | "_XLFN.SWITCH" => Some(Function::Switch),
            "TRUE" => Some(Function::True),
            "XOR" | "_XLFN.XOR" => Some(Function::Xor),

            "SIN" => Some(Function::Sin),
            "COS" => Some(Function::Cos),
            "TAN" => Some(Function::Tan),

            "ASIN" => Some(Function::Asin),
            "ACOS" => Some(Function::Acos),
            "ATAN" => Some(Function::Atan),

            "SINH" => Some(Function::Sinh),
            "COSH" => Some(Function::Cosh),
            "TANH" => Some(Function::Tanh),

            "ASINH" => Some(Function::Asinh),
            "ACOSH" => Some(Function::Acosh),
            "ATANH" => Some(Function::Atanh),

            "PI" => Some(Function::Pi),
            "ABS" => Some(Function::Abs),
            "SQRT" => Some(Function::Sqrt),
            "SQRTPI" => Some(Function::Sqrtpi),
            "POWER" => Some(Function::Power),
            "ATAN2" => Some(Function::Atan2),

            "MAX" => Some(Function::Max),
            "MIN" => Some(Function::Min),
            "PRODUCT" => Some(Function::Product),
            "RAND" => Some(Function::Rand),
            "RANDBETWEEN" => Some(Function::Randbetween),
            "ROUND" => Some(Function::Round),
            "ROUNDDOWN" => Some(Function::Rounddown),
            "ROUNDUP" => Some(Function::Roundup),
            "SUM" => Some(Function::Sum),
            "SUMIF" => Some(Function::Sumif),
            "SUMIFS" => Some(Function::Sumifs),

            // Lookup and Reference
            "CHOOSE" => Some(Function::Choose),
            "COLUMN" => Some(Function::Column),
            "COLUMNS" => Some(Function::Columns),
            "INDEX" => Some(Function::Index),
            "INDIRECT" => Some(Function::Indirect),
            "HLOOKUP" => Some(Function::Hlookup),
            "LOOKUP" => Some(Function::Lookup),
            "MATCH" => Some(Function::Match),
            "OFFSET" => Some(Function::Offset),
            "ROW" => Some(Function::Row),
            "ROWS" => Some(Function::Rows),
            "VLOOKUP" => Some(Function::Vlookup),
            "XLOOKUP" | "_XLFN.XLOOKUP" => Some(Function::Xlookup),

            "CONCATENATE" => Some(Function::Concatenate),
            "EXACT" => Some(Function::Exact),
            "VALUE" => Some(Function::Value),
            "T" => Some(Function::T),
            "VALUETOTEXT" | "_XLFN.VALUETOTEXT" => Some(Function::Valuetotext),
            "CONCAT" | "_XLFN.CONCAT" => Some(Function::Concat),
            "FIND" => Some(Function::Find),
            "LEFT" => Some(Function::Left),
            "LEN" => Some(Function::Len),
            "LOWER" => Some(Function::Lower),
            "MID" => Some(Function::Mid),
            "RIGHT" => Some(Function::Right),
            "SEARCH" => Some(Function::Search),
            "TEXT" => Some(Function::Text),
            "TRIM" => Some(Function::Trim),
            "UPPER" => Some(Function::Upper),

            "REPT" => Some(Function::Rept),
            "TEXTAFTER" | "_XLFN.TEXTAFTER" => Some(Function::Textafter),
            "TEXTBEFORE" | "_XLFN.TEXTBEFORE" => Some(Function::Textbefore),
            "TEXTJOIN" | "_XLFN.TEXTJOIN" => Some(Function::Textjoin),
            "SUBSTITUTE" => Some(Function::Substitute),

            "ISNUMBER" => Some(Function::Isnumber),
            "ISNONTEXT" => Some(Function::Isnontext),
            "ISTEXT" => Some(Function::Istext),
            "ISLOGICAL" => Some(Function::Islogical),
            "ISBLANK" => Some(Function::Isblank),
            "ISERR" => Some(Function::Iserr),
            "ISERROR" => Some(Function::Iserror),
            "ISNA" => Some(Function::Isna),
            "NA" => Some(Function::Na),
            "ISREF" => Some(Function::Isref),
            "ISODD" => Some(Function::Isodd),
            "ISEVEN" => Some(Function::Iseven),
            "ERROR.TYPE" => Some(Function::ErrorType),
            "ISFORMULA" | "_XLFN.ISFORMULA" => Some(Function::Isformula),
            "TYPE" => Some(Function::Type),
            "SHEET" | "_XLFN.SHEET" => Some(Function::Sheet),

            "AVERAGE" => Some(Function::Average),
            "AVERAGEA" => Some(Function::Averagea),
            "AVERAGEIF" => Some(Function::Averageif),
            "AVERAGEIFS" => Some(Function::Averageifs),
            "COUNT" => Some(Function::Count),
            "COUNTA" => Some(Function::Counta),
            "COUNTBLANK" => Some(Function::Countblank),
            "COUNTIF" => Some(Function::Countif),
            "COUNTIFS" => Some(Function::Countifs),
            "MAXIFS" | "_XLFN.MAXIFS" => Some(Function::Maxifs),
            "MINIFS" | "_XLFN.MINIFS" => Some(Function::Minifs),
            // Date and Time
            "YEAR" => Some(Function::Year),
            "DAY" => Some(Function::Day),
            "MONTH" => Some(Function::Month),
            "DATE" => Some(Function::Date),
            "EDATE" => Some(Function::Edate),
            "TODAY" => Some(Function::Today),
            "NOW" => Some(Function::Now),
            // Financial
            "PMT" => Some(Function::Pmt),
            "PV" => Some(Function::Pv),
            "RATE" => Some(Function::Rate),
            "NPER" => Some(Function::Nper),
            "FV" => Some(Function::Fv),
            "PPMT" => Some(Function::Ppmt),
            "IPMT" => Some(Function::Ipmt),
            "NPV" => Some(Function::Npv),
            "XNPV" => Some(Function::Xnpv),
            "MIRR" => Some(Function::Mirr),
            "IRR" => Some(Function::Irr),
            "XIRR" => Some(Function::Xirr),
            "ISPMT" => Some(Function::Ispmt),
            "RRI" | "_XLFN.RRI" => Some(Function::Rri),

            "SLN" => Some(Function::Sln),
            "SYD" => Some(Function::Syd),
            "NOMINAL" => Some(Function::Nominal),
            "EFFECT" => Some(Function::Effect),
            "PDURATION" | "_XLFN.PDURATION" => Some(Function::Pduration),

            "TBILLYIELD" => Some(Function::Tbillyield),
            "TBILLPRICE" => Some(Function::Tbillprice),
            "TBILLEQ" => Some(Function::Tbilleq),

            "DOLLARDE" => Some(Function::Dollarde),
            "DOLLARFR" => Some(Function::Dollarfr),

            "DDB" => Some(Function::Ddb),
            "DB" => Some(Function::Db),

            "CUMPRINC" => Some(Function::Cumprinc),
            "CUMIPMT" => Some(Function::Cumipmt),

            "BESSELI" => Some(Function::Besseli),
            "BESSELJ" => Some(Function::Besselj),
            "BESSELK" => Some(Function::Besselk),
            "BESSELY" => Some(Function::Bessely),
            "ERF" => Some(Function::Erf),
            "ERF.PRECISE" | "_XLFN.ERF.PRECISE" => Some(Function::ErfPrecise),
            "ERFC" => Some(Function::Erfc),
            "ERFC.PRECISE" | "_XLFN.ERFC.PRECISE" => Some(Function::ErfcPrecise),
            "BIN2DEC" => Some(Function::Bin2dec),
            "BIN2HEX" => Some(Function::Bin2hex),
            "BIN2OCT" => Some(Function::Bin2oct),
            "DEC2BIN" => Some(Function::Dec2Bin),
            "DEC2HEX" => Some(Function::Dec2hex),
            "DEC2OCT" => Some(Function::Dec2oct),
            "HEX2BIN" => Some(Function::Hex2bin),
            "HEX2DEC" => Some(Function::Hex2dec),
            "HEX2OCT" => Some(Function::Hex2oct),
            "OCT2BIN" => Some(Function::Oct2bin),
            "OCT2DEC" => Some(Function::Oct2dec),
            "OCT2HEX" => Some(Function::Oct2hex),
            "BITAND" | "_XLFN.BITAND" => Some(Function::Bitand),
            "BITLSHIFT" | "_XLFN.BITLSHIFT" => Some(Function::Bitlshift),
            "BITOR" | "_XLFN.BITOR" => Some(Function::Bitor),
            "BITRSHIFT" | "_XLFN.BITRSHIFT" => Some(Function::Bitrshift),
            "BITXOR" | "_XLFN.BITXOR" => Some(Function::Bitxor),
            "COMPLEX" => Some(Function::Complex),
            "IMABS" => Some(Function::Imabs),
            "IMAGINARY" => Some(Function::Imaginary),
            "IMARGUMENT" => Some(Function::Imargument),
            "IMCONJUGATE" => Some(Function::Imconjugate),
            "IMCOS" => Some(Function::Imcos),
            "IMCOSH" | "_XLFN.IMCOSH" => Some(Function::Imcosh),
            "IMCOT" | "_XLFN.IMCOT" => Some(Function::Imcot),
            "IMCSC" | "_XLFN.IMCSC" => Some(Function::Imcsc),
            "IMCSCH" | "_XLFN.IMCSCH" => Some(Function::Imcsch),
            "IMDIV" => Some(Function::Imdiv),
            "IMEXP" => Some(Function::Imexp),
            "IMLN" => Some(Function::Imln),
            "IMLOG10" => Some(Function::Imlog10),
            "IMLOG2" => Some(Function::Imlog2),
            "IMPOWER" => Some(Function::Impower),
            "IMPRODUCT" => Some(Function::Improduct),
            "IMREAL" => Some(Function::Imreal),
            "IMSEC" | "_XLFN.IMSEC" => Some(Function::Imsec),
            "IMSECH" | "_XLFN.IMSECH" => Some(Function::Imsech),
            "IMSIN" => Some(Function::Imsin),
            "IMSINH" | "_XLFN.IMSINH" => Some(Function::Imsinh),
            "IMSQRT" => Some(Function::Imsqrt),
            "IMSUB" => Some(Function::Imsub),
            "IMSUM" => Some(Function::Imsum),
            "IMTAN" | "_XLFN.IMTAN" => Some(Function::Imtan),
            "CONVERT" => Some(Function::Convert),
            "DELTA" => Some(Function::Delta),
            "GESTEP" => Some(Function::Gestep),

            "SUBTOTAL" => Some(Function::Subtotal),
            _ => None,
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Function::And => write!(f, "AND"),
            Function::False => write!(f, "FALSE"),
            Function::If => write!(f, "IF"),
            Function::Iferror => write!(f, "IFERROR"),
            Function::Ifna => write!(f, "IFNA"),
            Function::Ifs => write!(f, "IFS"),
            Function::Not => write!(f, "NOT"),
            Function::Or => write!(f, "OR"),
            Function::Switch => write!(f, "SWITCH"),
            Function::True => write!(f, "TRUE"),
            Function::Xor => write!(f, "XOR"),
            Function::Sin => write!(f, "SIN"),
            Function::Cos => write!(f, "COS"),
            Function::Tan => write!(f, "TAN"),
            Function::Asin => write!(f, "ASIN"),
            Function::Acos => write!(f, "ACOS"),
            Function::Atan => write!(f, "ATAN"),
            Function::Sinh => write!(f, "SINH"),
            Function::Cosh => write!(f, "COSH"),
            Function::Tanh => write!(f, "TANH"),
            Function::Asinh => write!(f, "ASINH"),
            Function::Acosh => write!(f, "ACOSH"),
            Function::Atanh => write!(f, "ATANH"),
            Function::Abs => write!(f, "ABS"),
            Function::Pi => write!(f, "PI"),
            Function::Sqrt => write!(f, "SQRT"),
            Function::Sqrtpi => write!(f, "SQRTPI"),
            Function::Atan2 => write!(f, "ATAN2"),
            Function::Power => write!(f, "POWER"),
            Function::Max => write!(f, "MAX"),
            Function::Min => write!(f, "MIN"),
            Function::Product => write!(f, "PRODUCT"),
            Function::Rand => write!(f, "RAND"),
            Function::Randbetween => write!(f, "RANDBETWEEN"),
            Function::Round => write!(f, "ROUND"),
            Function::Rounddown => write!(f, "ROUNDDOWN"),
            Function::Roundup => write!(f, "ROUNDUP"),
            Function::Sum => write!(f, "SUM"),
            Function::Sumif => write!(f, "SUMIF"),
            Function::Sumifs => write!(f, "SUMIFS"),
            Function::Choose => write!(f, "CHOOSE"),
            Function::Column => write!(f, "COLUMN"),
            Function::Columns => write!(f, "COLUMNS"),
            Function::Index => write!(f, "INDEX"),
            Function::Indirect => write!(f, "INDIRECT"),
            Function::Hlookup => write!(f, "HLOOKUP"),
            Function::Lookup => write!(f, "LOOKUP"),
            Function::Match => write!(f, "MATCH"),
            Function::Offset => write!(f, "OFFSET"),
            Function::Row => write!(f, "ROW"),
            Function::Rows => write!(f, "ROWS"),
            Function::Vlookup => write!(f, "VLOOKUP"),
            Function::Xlookup => write!(f, "XLOOKUP"),
            Function::Concatenate => write!(f, "CONCATENATE"),
            Function::Exact => write!(f, "EXACT"),
            Function::Value => write!(f, "VALUE"),
            Function::T => write!(f, "T"),
            Function::Valuetotext => write!(f, "VALUETOTEXT"),
            Function::Concat => write!(f, "CONCAT"),
            Function::Find => write!(f, "FIND"),
            Function::Left => write!(f, "LEFT"),
            Function::Len => write!(f, "LEN"),
            Function::Lower => write!(f, "LOWER"),
            Function::Mid => write!(f, "MID"),
            Function::Right => write!(f, "RIGHT"),
            Function::Search => write!(f, "SEARCH"),
            Function::Text => write!(f, "TEXT"),
            Function::Trim => write!(f, "TRIM"),
            Function::Upper => write!(f, "UPPER"),
            Function::Isnumber => write!(f, "ISNUMBER"),
            Function::Isnontext => write!(f, "ISNONTEXT"),
            Function::Istext => write!(f, "ISTEXT"),
            Function::Islogical => write!(f, "ISLOGICAL"),
            Function::Isblank => write!(f, "ISBLANK"),
            Function::Iserr => write!(f, "ISERR"),
            Function::Iserror => write!(f, "ISERROR"),
            Function::Isna => write!(f, "ISNA"),
            Function::Na => write!(f, "NA"),
            Function::Isref => write!(f, "ISREF"),
            Function::Isodd => write!(f, "ISODD"),
            Function::Iseven => write!(f, "ISEVEN"),
            Function::ErrorType => write!(f, "ERROR.TYPE"),
            Function::Isformula => write!(f, "ISFORMULA"),
            Function::Type => write!(f, "TYPE"),
            Function::Sheet => write!(f, "SHEET"),

            Function::Average => write!(f, "AVERAGE"),
            Function::Averagea => write!(f, "AVERAGEA"),
            Function::Averageif => write!(f, "AVERAGEIF"),
            Function::Averageifs => write!(f, "AVERAGEIFS"),
            Function::Count => write!(f, "COUNT"),
            Function::Counta => write!(f, "COUNTA"),
            Function::Countblank => write!(f, "COUNTBLANK"),
            Function::Countif => write!(f, "COUNTIF"),
            Function::Countifs => write!(f, "COUNTIFS"),
            Function::Maxifs => write!(f, "MAXIFS"),
            Function::Minifs => write!(f, "MINIFS"),
            Function::Year => write!(f, "YEAR"),
            Function::Day => write!(f, "DAY"),
            Function::Month => write!(f, "MONTH"),
            Function::Date => write!(f, "DATE"),
            Function::Edate => write!(f, "EDATE"),
            Function::Today => write!(f, "TODAY"),
            Function::Now => write!(f, "NOW"),
            Function::Pmt => write!(f, "PMT"),
            Function::Pv => write!(f, "PV"),
            Function::Rate => write!(f, "RATE"),
            Function::Nper => write!(f, "NPER"),
            Function::Fv => write!(f, "FV"),
            Function::Ppmt => write!(f, "PPMT"),
            Function::Ipmt => write!(f, "IPMT"),
            Function::Npv => write!(f, "NPV"),
            Function::Mirr => write!(f, "MIRR"),
            Function::Irr => write!(f, "IRR"),
            Function::Xirr => write!(f, "XIRR"),
            Function::Xnpv => write!(f, "XNPV"),
            Function::Rept => write!(f, "REPT"),
            Function::Textafter => write!(f, "TEXTAFTER"),
            Function::Textbefore => write!(f, "TEXTBEFORE"),
            Function::Textjoin => write!(f, "TEXTJOIN"),
            Function::Substitute => write!(f, "SUBSTITUTE"),
            Function::Ispmt => write!(f, "ISPMT"),
            Function::Rri => write!(f, "RRI"),
            Function::Sln => write!(f, "SLN"),
            Function::Syd => write!(f, "SYD"),
            Function::Nominal => write!(f, "NOMINAL"),
            Function::Effect => write!(f, "EFFECT"),
            Function::Pduration => write!(f, "PDURATION"),
            Function::Tbillyield => write!(f, "TBILLYIELD"),
            Function::Tbillprice => write!(f, "TBILLPRICE"),
            Function::Tbilleq => write!(f, "TBILLEQ"),
            Function::Dollarde => write!(f, "DOLLARDE"),
            Function::Dollarfr => write!(f, "DOLLARFR"),
            Function::Ddb => write!(f, "DDB"),
            Function::Db => write!(f, "DB"),
            Function::Cumprinc => write!(f, "CUMPRINC"),
            Function::Cumipmt => write!(f, "CUMIPMT"),
            Function::Besseli => write!(f, "BESSELI"),
            Function::Besselj => write!(f, "BESSELJ"),
            Function::Besselk => write!(f, "BESSELK"),
            Function::Bessely => write!(f, "BESSELY"),
            Function::Erf => write!(f, "ERF"),
            Function::ErfPrecise => write!(f, "ERF.PRECISE"),
            Function::Erfc => write!(f, "ERFC"),
            Function::ErfcPrecise => write!(f, "ERFC.PRECISE"),
            Function::Bin2dec => write!(f, "BIN2DEC"),
            Function::Bin2hex => write!(f, "BIN2HEX"),
            Function::Bin2oct => write!(f, "BIN2OCT"),
            Function::Dec2Bin => write!(f, "DEC2BIN"),
            Function::Dec2hex => write!(f, "DEC2HEX"),
            Function::Dec2oct => write!(f, "DEC2OCT"),
            Function::Hex2bin => write!(f, "HEX2BIN"),
            Function::Hex2dec => write!(f, "HEX2DEC"),
            Function::Hex2oct => write!(f, "HEX2OCT"),
            Function::Oct2bin => write!(f, "OCT2BIN"),
            Function::Oct2dec => write!(f, "OCT2DEC"),
            Function::Oct2hex => write!(f, "OCT2HEX"),
            Function::Bitand => write!(f, "BITAND"),
            Function::Bitlshift => write!(f, "BITLSHIFT"),
            Function::Bitor => write!(f, "BITOR"),
            Function::Bitrshift => write!(f, "BITRSHIFT"),
            Function::Bitxor => write!(f, "BITXOR"),
            Function::Complex => write!(f, "COMPLEX"),
            Function::Imabs => write!(f, "IMABS"),
            Function::Imaginary => write!(f, "IMAGINARY"),
            Function::Imargument => write!(f, "IMARGUMENT"),
            Function::Imconjugate => write!(f, "IMCONJUGATE"),
            Function::Imcos => write!(f, "IMCOS"),
            Function::Imcosh => write!(f, "IMCOSH"),
            Function::Imcot => write!(f, "IMCOT"),
            Function::Imcsc => write!(f, "IMCSC"),
            Function::Imcsch => write!(f, "IMCSCH"),
            Function::Imdiv => write!(f, "IMDIV"),
            Function::Imexp => write!(f, "IMEXP"),
            Function::Imln => write!(f, "IMLN"),
            Function::Imlog10 => write!(f, "IMLOG10"),
            Function::Imlog2 => write!(f, "IMLOG2"),
            Function::Impower => write!(f, "IMPOWER"),
            Function::Improduct => write!(f, "IMPRODUCT"),
            Function::Imreal => write!(f, "IMREAL"),
            Function::Imsec => write!(f, "IMSEC"),
            Function::Imsech => write!(f, "IMSECH"),
            Function::Imsin => write!(f, "IMSIN"),
            Function::Imsinh => write!(f, "IMSINH"),
            Function::Imsqrt => write!(f, "IMSQRT"),
            Function::Imsub => write!(f, "IMSUB"),
            Function::Imsum => write!(f, "IMSUM"),
            Function::Imtan => write!(f, "IMTAN"),
            Function::Convert => write!(f, "CONVERT"),
            Function::Delta => write!(f, "DELTA"),
            Function::Gestep => write!(f, "GESTEP"),

            Function::Subtotal => write!(f, "SUBTOTAL"),
        }
    }
}

impl Model {
    pub(crate) fn evaluate_function(
        &mut self,
        kind: &Function,
        args: &[Node],
        cell: CellReference,
    ) -> CalcResult {
        match kind {
            // Logical
            Function::And => self.fn_and(args, cell),
            Function::False => CalcResult::Boolean(false),
            Function::If => self.fn_if(args, cell),
            Function::Iferror => self.fn_iferror(args, cell),
            Function::Ifna => self.fn_ifna(args, cell),
            Function::Ifs => self.fn_ifs(args, cell),
            Function::Not => self.fn_not(args, cell),
            Function::Or => self.fn_or(args, cell),
            Function::Switch => self.fn_switch(args, cell),
            Function::True => CalcResult::Boolean(true),
            Function::Xor => self.fn_xor(args, cell),
            // Math and trigonometry
            Function::Sin => self.fn_sin(args, cell),
            Function::Cos => self.fn_cos(args, cell),
            Function::Tan => self.fn_tan(args, cell),

            Function::Asin => self.fn_asin(args, cell),
            Function::Acos => self.fn_acos(args, cell),
            Function::Atan => self.fn_atan(args, cell),

            Function::Sinh => self.fn_sinh(args, cell),
            Function::Cosh => self.fn_cosh(args, cell),
            Function::Tanh => self.fn_tanh(args, cell),

            Function::Asinh => self.fn_asinh(args, cell),
            Function::Acosh => self.fn_acosh(args, cell),
            Function::Atanh => self.fn_atanh(args, cell),

            Function::Pi => self.fn_pi(args, cell),
            Function::Abs => self.fn_abs(args, cell),

            Function::Sqrt => self.fn_sqrt(args, cell),
            Function::Sqrtpi => self.fn_sqrtpi(args, cell),
            Function::Atan2 => self.fn_atan2(args, cell),
            Function::Power => self.fn_power(args, cell),

            Function::Max => self.fn_max(args, cell),
            Function::Min => self.fn_min(args, cell),
            Function::Product => self.fn_product(args, cell),
            Function::Rand => self.fn_rand(args, cell),
            Function::Randbetween => self.fn_randbetween(args, cell),
            Function::Round => self.fn_round(args, cell),
            Function::Rounddown => self.fn_rounddown(args, cell),
            Function::Roundup => self.fn_roundup(args, cell),
            Function::Sum => self.fn_sum(args, cell),
            Function::Sumif => self.fn_sumif(args, cell),
            Function::Sumifs => self.fn_sumifs(args, cell),

            // Lookup and Reference
            Function::Choose => self.fn_choose(args, cell),
            Function::Column => self.fn_column(args, cell),
            Function::Columns => self.fn_columns(args, cell),
            Function::Index => self.fn_index(args, cell),
            Function::Indirect => self.fn_indirect(args, cell),
            Function::Hlookup => self.fn_hlookup(args, cell),
            Function::Lookup => self.fn_lookup(args, cell),
            Function::Match => self.fn_match(args, cell),
            Function::Offset => self.fn_offset(args, cell),
            Function::Row => self.fn_row(args, cell),
            Function::Rows => self.fn_rows(args, cell),
            Function::Vlookup => self.fn_vlookup(args, cell),
            Function::Xlookup => self.fn_xlookup(args, cell),
            // Text
            Function::Concatenate => self.fn_concatenate(args, cell),
            Function::Exact => self.fn_exact(args, cell),
            Function::Value => self.fn_value(args, cell),
            Function::T => self.fn_t(args, cell),
            Function::Valuetotext => self.fn_valuetotext(args, cell),
            Function::Concat => self.fn_concat(args, cell),
            Function::Find => self.fn_find(args, cell),
            Function::Left => self.fn_left(args, cell),
            Function::Len => self.fn_len(args, cell),
            Function::Lower => self.fn_lower(args, cell),
            Function::Mid => self.fn_mid(args, cell),
            Function::Right => self.fn_right(args, cell),
            Function::Search => self.fn_search(args, cell),
            Function::Text => self.fn_text(args, cell),
            Function::Trim => self.fn_trim(args, cell),
            Function::Upper => self.fn_upper(args, cell),
            // Information
            Function::Isnumber => self.fn_isnumber(args, cell),
            Function::Isnontext => self.fn_isnontext(args, cell),
            Function::Istext => self.fn_istext(args, cell),
            Function::Islogical => self.fn_islogical(args, cell),
            Function::Isblank => self.fn_isblank(args, cell),
            Function::Iserr => self.fn_iserr(args, cell),
            Function::Iserror => self.fn_iserror(args, cell),
            Function::Isna => self.fn_isna(args, cell),
            Function::Na => CalcResult::new_error(Error::NA, cell, "".to_string()),
            Function::Isref => self.fn_isref(args, cell),
            Function::Isodd => self.fn_isodd(args, cell),
            Function::Iseven => self.fn_iseven(args, cell),
            Function::ErrorType => self.fn_errortype(args, cell),
            Function::Isformula => self.fn_isformula(args, cell),
            Function::Type => self.fn_type(args, cell),
            Function::Sheet => self.fn_sheet(args, cell),
            // Statistical
            Function::Average => self.fn_average(args, cell),
            Function::Averagea => self.fn_averagea(args, cell),
            Function::Averageif => self.fn_averageif(args, cell),
            Function::Averageifs => self.fn_averageifs(args, cell),
            Function::Count => self.fn_count(args, cell),
            Function::Counta => self.fn_counta(args, cell),
            Function::Countblank => self.fn_countblank(args, cell),
            Function::Countif => self.fn_countif(args, cell),
            Function::Countifs => self.fn_countifs(args, cell),
            Function::Maxifs => self.fn_maxifs(args, cell),
            Function::Minifs => self.fn_minifs(args, cell),
            // Date and Time
            Function::Year => self.fn_year(args, cell),
            Function::Day => self.fn_day(args, cell),
            Function::Month => self.fn_month(args, cell),
            Function::Date => self.fn_date(args, cell),
            Function::Edate => self.fn_edate(args, cell),
            Function::Today => self.fn_today(args, cell),
            Function::Now => self.fn_now(args, cell),
            // Financial
            Function::Pmt => self.fn_pmt(args, cell),
            Function::Pv => self.fn_pv(args, cell),
            Function::Rate => self.fn_rate(args, cell),
            Function::Nper => self.fn_nper(args, cell),
            Function::Fv => self.fn_fv(args, cell),
            Function::Ppmt => self.fn_ppmt(args, cell),
            Function::Ipmt => self.fn_ipmt(args, cell),
            Function::Npv => self.fn_npv(args, cell),
            Function::Mirr => self.fn_mirr(args, cell),
            Function::Irr => self.fn_irr(args, cell),
            Function::Xirr => self.fn_xirr(args, cell),
            Function::Xnpv => self.fn_xnpv(args, cell),
            Function::Rept => self.fn_rept(args, cell),
            Function::Textafter => self.fn_textafter(args, cell),
            Function::Textbefore => self.fn_textbefore(args, cell),
            Function::Textjoin => self.fn_textjoin(args, cell),
            Function::Substitute => self.fn_substitute(args, cell),
            Function::Ispmt => self.fn_ispmt(args, cell),
            Function::Rri => self.fn_rri(args, cell),
            Function::Sln => self.fn_sln(args, cell),
            Function::Syd => self.fn_syd(args, cell),
            Function::Nominal => self.fn_nominal(args, cell),
            Function::Effect => self.fn_effect(args, cell),
            Function::Pduration => self.fn_pduration(args, cell),
            Function::Tbillyield => self.fn_tbillyield(args, cell),
            Function::Tbillprice => self.fn_tbillprice(args, cell),
            Function::Tbilleq => self.fn_tbilleq(args, cell),
            Function::Dollarde => self.fn_dollarde(args, cell),
            Function::Dollarfr => self.fn_dollarfr(args, cell),
            Function::Ddb => self.fn_ddb(args, cell),
            Function::Db => self.fn_db(args, cell),
            Function::Cumprinc => self.fn_cumprinc(args, cell),
            Function::Cumipmt => self.fn_cumipmt(args, cell),
            // Engineering
            Function::Besseli => self.fn_besseli(args, cell),
            Function::Besselj => self.fn_besselj(args, cell),
            Function::Besselk => self.fn_besselk(args, cell),
            Function::Bessely => self.fn_bessely(args, cell),
            Function::Erf => self.fn_erf(args, cell),
            Function::ErfPrecise => self.fn_erfprecise(args, cell),
            Function::Erfc => self.fn_erfc(args, cell),
            Function::ErfcPrecise => self.fn_erfcprecise(args, cell),
            Function::Bin2dec => self.fn_bin2dec(args, cell),
            Function::Bin2hex => self.fn_bin2hex(args, cell),
            Function::Bin2oct => self.fn_bin2oct(args, cell),
            Function::Dec2Bin => self.fn_dec2bin(args, cell),
            Function::Dec2hex => self.fn_dec2hex(args, cell),
            Function::Dec2oct => self.fn_dec2oct(args, cell),
            Function::Hex2bin => self.fn_hex2bin(args, cell),
            Function::Hex2dec => self.fn_hex2dec(args, cell),
            Function::Hex2oct => self.fn_hex2oct(args, cell),
            Function::Oct2bin => self.fn_oct2bin(args, cell),
            Function::Oct2dec => self.fn_oct2dec(args, cell),
            Function::Oct2hex => self.fn_oct2hex(args, cell),
            Function::Bitand => self.fn_bitand(args, cell),
            Function::Bitlshift => self.fn_bitlshift(args, cell),
            Function::Bitor => self.fn_bitor(args, cell),
            Function::Bitrshift => self.fn_bitrshift(args, cell),
            Function::Bitxor => self.fn_bitxor(args, cell),
            Function::Complex => self.fn_complex(args, cell),
            Function::Imabs => self.fn_imabs(args, cell),
            Function::Imaginary => self.fn_imaginary(args, cell),
            Function::Imargument => self.fn_imargument(args, cell),
            Function::Imconjugate => self.fn_imconjugate(args, cell),
            Function::Imcos => self.fn_imcos(args, cell),
            Function::Imcosh => self.fn_imcosh(args, cell),
            Function::Imcot => self.fn_imcot(args, cell),
            Function::Imcsc => self.fn_imcsc(args, cell),
            Function::Imcsch => self.fn_imcsch(args, cell),
            Function::Imdiv => self.fn_imdiv(args, cell),
            Function::Imexp => self.fn_imexp(args, cell),
            Function::Imln => self.fn_imln(args, cell),
            Function::Imlog10 => self.fn_imlog10(args, cell),
            Function::Imlog2 => self.fn_imlog2(args, cell),
            Function::Impower => self.fn_impower(args, cell),
            Function::Improduct => self.fn_improduct(args, cell),
            Function::Imreal => self.fn_imreal(args, cell),
            Function::Imsec => self.fn_imsec(args, cell),
            Function::Imsech => self.fn_imsech(args, cell),
            Function::Imsin => self.fn_imsin(args, cell),
            Function::Imsinh => self.fn_imsinh(args, cell),
            Function::Imsqrt => self.fn_imsqrt(args, cell),
            Function::Imsub => self.fn_imsub(args, cell),
            Function::Imsum => self.fn_imsum(args, cell),
            Function::Imtan => self.fn_imtan(args, cell),
            Function::Convert => self.fn_convert(args, cell),
            Function::Delta => self.fn_delta(args, cell),
            Function::Gestep => self.fn_gestep(args, cell),

            Function::Subtotal => self.fn_subtotal(args, cell),
        }
    }
}
