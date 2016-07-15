#![feature(box_syntax, box_patterns)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate rotor;
extern crate thread_scoped;
extern crate nix;
extern crate httparse;
extern crate rustc_serialize as serialize;
extern crate tempfile;

#[macro_use]
mod macros;
mod logger;
mod env;
mod dispatcher;
mod docker;
mod writer;
mod config;

use std::process::exit;


fn main() {
    logger::init().unwrap();
    
    try_or_exit!(dispatcher::start(), "Error while running: {}");
}
