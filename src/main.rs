#[macro_use]
extern crate clap;
extern crate cron_gate;

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
                println!("Expression: {:?}", exp);
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}
