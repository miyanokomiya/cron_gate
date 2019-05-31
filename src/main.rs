#[macro_use]
extern crate clap;
extern crate cron_gate;

use chrono::Local;
use clap::Arg;
use cron_gate::expression::Expression;

fn main() {
    let app = app_from_crate!()
        .arg(Arg::from_usage(
            "<expression> 'Cron Expression: * * * 7 * command'",
        ))
        .arg(
            Arg::with_name("number")
                .help("Displayed number")
                .short("n")
                .long("number")
                .takes_value(true)
                .default_value("10"),
        );

    let matches = app.get_matches();

    let mut number = 10;
    if let Some(n_str) = matches.value_of("number") {
        if let Ok(n) = n_str.parse::<usize>() {
            number = n;
        }
    }

    if let Some(o) = matches.value_of("expression") {
        match Expression::new(o) {
            Ok(exp) => {
                let now = Local::now();
                let datetimes = exp.earler_excuting_datetimes(now, number);
                for (i, dt) in datetimes.iter().enumerate() {
                    println!("{}: {}", i + 1, dt.format("%m/%d %H:%M").to_string());
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}
