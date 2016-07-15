
use std;
use std::sync::{Mutex, Condvar};
use std::str::Utf8Error;
use std::io::Error as IoError;
use rotor::{SpawnError};
use httparse::Error as HttparseError;
use httparse::InvalidChunkSize;
use serialize::json::ParserError as JsonError;

use docker;
use writer;

pub fn start() -> Result<()> {
    let ctx = Context::new();
    {
        let _docker = docker::start(&ctx);
        let _writer = writer::start(&ctx);
    }
    Ok(())
}

pub struct Context {
    pub changed: Mutex<bool>,
    pub lock: Condvar,
}

impl Context {
    fn new() -> Context {
        Context {
            changed: Mutex::new(false),
            lock: Condvar::new(),
        }
    }
}


quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(e: IoError) {
            from()
            description("IO error")
            display("IO error {}", e)
            cause(e)
        }
        Spawn(e: SpawnError<()>) {
            from()
        }
        Httparse(e: HttparseError) {
            from()
        }
        ChunkSize {
            from(InvalidChunkSize)
        }
        Json(e: JsonError) {
            from()
        }
        Utf8(e: Utf8Error) {
            from()
        }
    }
}
pub type Result<T> = std::result::Result<T, Error>;
