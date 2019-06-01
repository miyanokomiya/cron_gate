#[macro_use]
extern crate clap;
extern crate cron_gate;

use chrono::offset::TimeZone;
use chrono::Local;
use clap::Arg;
use cron_gate::expression::{Expression, DATE_FORMAT};

fn main() {
    let app = app_from_crate!()
        .arg(
            Arg::with_name("expression")
                .help("Cron Expression '* * * 7 * [command]'")
                .required(true),
        )
        .arg(
            Arg::with_name("after")
                .help("Dates after 'Y/m/d H:M:S'")
                .short("a")
                .long("after")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("number")
                .help("Displayed number")
                .short("n")
                .long("number")
                .takes_value(true)
                .default_value("10"),
        );

    let matches = app.get_matches();

    let mut after = Local::now();
    if let Some(a_str) = matches.value_of("after") {
        match Local.datetime_from_str(a_str, DATE_FORMAT) {
            Ok(a) => after = a,
            Err(e) => {
                eprintln!("Invalid -a value: '{}'", a_str);
                panic!(e);
            }
        }
    }

    let mut number = 10;
    if let Some(n_str) = matches.value_of("number") {
        if let Ok(n) = n_str.parse::<usize>() {
            number = n;
        }
    }

    if let Some(o) = matches.value_of("expression") {
        match Expression::new(o) {
            Ok(exp) => {
                let datetimes = exp.executing_dates(after, number);
                for dt in datetimes {
                    println!("{}", dt);
                }
            }
            Err(e) => panic!(e),
        }
    }
}
