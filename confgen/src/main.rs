#![feature(box_syntax, box_patterns)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate rotor;
extern crate rotor_tools;
extern crate nix;
extern crate httparse;
extern crate rustc_serialize as serialize;

mod logger;
mod env;
mod dispatcher;
mod docker;

use std::process::exit;

macro_rules! try_or_exit {
    ($expr:expr, $($arg:tt)*) => (match $expr {
        Ok(e) => e,
        Err(ref e) => {
            error!($($arg)*, e);
            exit(1);
        }
    })
}

fn main() {
    logger::init().unwrap();
    
    try_or_exit!(dispatcher::start(), "Error while running: {}");
}
