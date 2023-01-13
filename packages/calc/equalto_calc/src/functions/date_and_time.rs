use chrono::Datelike;
use chrono::Months;
use chrono::NaiveDate;

use chrono::NaiveDateTime;
use chrono::TimeZone;

use crate::{
    calc_result::{CalcResult, CellReference},
    constants::EXCEL_DATE_BASE,
    expressions::parser::Node,
    expressions::token::Error,
    formatter::dates::from_excel_date,
    model::Model,
};

impl Model {
    pub(crate) fn fn_day(&mut self, args: &[Node], cell: CellReference) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i64;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Function DAY parameter 1 value is negative. It should be positive or zero.".to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };
        let date = from_excel_date(serial_number);
        let day = date.day() as f64;
        CalcResult::Number(day)
    }

    pub(crate) fn fn_month(&mut self, args: &[Node], cell: CellReference) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i64;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Function MONTH parameter 1 value is negative. It should be positive or zero.".to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };
        let date = from_excel_date(serial_number);
        let month = date.month() as f64;
        CalcResult::Number(month)
    }

    // year, month, day
    pub(crate) fn fn_date(&mut self, args: &[Node], cell: CellReference) -> CalcResult {
        let args_count = args.len();
        if args_count != 3 {
            return CalcResult::new_args_number_error(cell);
        }
        let year = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i32;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Out of range parameters for date".to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };
        let month = match self.get_number(&args[1], cell) {
            Ok(c) => {
                let t = c.floor();
                if t < 0.0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Out of range parameters for date".to_string(),
                    };
                }
                t as u32
            }
            Err(s) => return s,
        };
        let day = match self.get_number(&args[2], cell) {
            Ok(c) => {
                let t = c.floor();
                if t < 0.0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Out of range parameters for date".to_string(),
                    };
                }
                t as u32
            }
            Err(s) => return s,
        };
        let serial_number = match NaiveDate::from_ymd_opt(year, month, day) {
            Some(native_date) => native_date.num_days_from_ce() - EXCEL_DATE_BASE,
            None => {
                return CalcResult::Error {
                    error: Error::NUM,
                    origin: cell,
                    message: "Out of range parameters for date".to_string(),
                };
            }
        };
        CalcResult::Number(serial_number as f64)
    }

    pub(crate) fn fn_year(&mut self, args: &[Node], cell: CellReference) -> CalcResult {
        let args_count = args.len();
        if args_count != 1 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i64;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Function YEAR parameter 1 value is negative. It should be positive or zero.".to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };
        let date = from_excel_date(serial_number);
        let year = date.year() as f64;
        CalcResult::Number(year)
    }

    pub(crate) fn fn_today(&mut self, args: &[Node], cell: CellReference) -> CalcResult {
        let args_count = args.len();
        if args_count != 0 {
            return CalcResult::new_args_number_error(cell);
        }
        // milliseconds since January 1, 1970 00:00:00 UTC.
        let milliseconds = (self.env.get_milliseconds_since_epoch)();
        let seconds = milliseconds / 1000;
        let dt = NaiveDateTime::from_timestamp_opt(seconds, 0)
            .expect("problem with chrono::NaiveDateTime");
        let local_time = self.tz.from_utc_datetime(&dt);
        let days_from_1900 = local_time.num_days_from_ce() - EXCEL_DATE_BASE;

        CalcResult::Number(days_from_1900 as f64)
    }

    // date, months
    pub(crate) fn fn_edate(&mut self, args: &[Node], cell: CellReference) -> CalcResult {
        let args_count = args.len();
        if args_count != 2 {
            return CalcResult::new_args_number_error(cell);
        }
        let serial_number = match self.get_number(&args[0], cell) {
            Ok(c) => {
                let t = c.floor() as i64;
                if t < 0 {
                    return CalcResult::Error {
                        error: Error::NUM,
                        origin: cell,
                        message: "Parameter 1 value is negative. It should be positive or zero."
                            .to_string(),
                    };
                }
                t
            }
            Err(s) => return s,
        };

        let months = match self.get_number(&args[1], cell) {
            Ok(c) => {
                let t = c.trunc();
                t as i32
            }
            Err(s) => return s,
        };

        let months_abs = months.unsigned_abs();

        let native_date = if months > 0 {
            from_excel_date(serial_number) + Months::new(months_abs)
        } else {
            from_excel_date(serial_number) - Months::new(months_abs)
        };

        let serial_number = native_date.num_days_from_ce() - EXCEL_DATE_BASE;
        if serial_number < 0 {
            return CalcResult::Error {
                error: Error::NUM,
                origin: cell,
                message: "EDATE out of bounds".to_string(),
            };
        }
        CalcResult::Number(serial_number as f64)
    }
}
