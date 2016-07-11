
use std;
use std::vec::Vec;
use std::sync::{Arc};
use std::io::Error as IoError;
use rotor::{SpawnError};

use docker;

macro_rules! join {
    ($($h:expr),*) => {
        {
            let mut handles = Vec::new();
            $(
                handles.push($h);
            )*
            for h in handles {
                h.join().unwrap();
            }
        }
    }
}

pub fn start() -> Result<()> {
    let ctx = Arc::new(Context::new());
    join![
        docker::start(ctx.clone())
    ];
    Ok(())
}

pub struct Context {
}

impl Context {
    fn new() -> Context {
        Context {}
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
    }
}
pub type Result<T> = std::result::Result<T, Error>;
