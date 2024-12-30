mod config;
mod worker;

use std::env;
use worker::run_worker;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config.json>", args[0]);
        std::process::exit(1);
    }

    run_worker(&args[1]);
}
