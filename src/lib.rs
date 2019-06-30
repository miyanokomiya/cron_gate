extern crate wasm_bindgen;
pub mod expression;

use chrono::offset::TimeZone;
use chrono::Local;
use expression::{Expression, DATE_FORMAT};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_datetimes(text: &str, after_str: &str, number: i32) -> String {
    let after;
    match Local.datetime_from_str(after_str, DATE_FORMAT) {
        Ok(a) => after = a,
        Err(e) => {
            eprintln!("Invalid --after value: '{}'", after_str);
            return format!("{}, {}", after_str, e.to_string());
        }
    }

    match Expression::new(text) {
        Ok(exp) => {
            let datetimes = exp.executing_dates(after, number as usize);
            let vec: Vec<String> = datetimes.iter().map(|d| d.to_string()).collect();
            return vec.join("\n");
        }
        Err(e) => {
            return e.to_string();
        }
    }
}
