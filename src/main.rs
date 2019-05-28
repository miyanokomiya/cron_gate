#[macro_use]
extern crate clap;
extern crate cron_gate;

use chrono::Local;
use clap::Arg;
use cron_gate::expression::Expression;

fn main() {
    let app = app_from_crate!().arg(Arg::from_usage(
        "<expression> 'Cron Expression: * * * ? * command'",
    ));

    let matches = app.get_matches();

    if let Some(o) = matches.value_of("expression") {
        match Expression::new(o) {
            Ok(exp) => {
                let now = Local::now();
                let datetimes = exp.earler_excuting_datetimes(now, 10);
                for (i, dt) in datetimes.iter().enumerate() {
                    println!("{}: {}", i + 1, dt.format("%m/%d %H:%M").to_string());
                }
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}
