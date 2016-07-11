
use std;
use std::io::{Write, Read};
use std::time::Duration;
use std::thread::{self, JoinHandle};
use std::sync::Arc;
use rotor::mio::unix::UnixStream;
use rotor::{Machine, Response, Scope, EarlyScope, EventSet, Loop, Config, PollOpt};
use rotor::void::{Void, unreachable};
use httparse;
use serialize::json::Json;

use dispatcher::{Context, Result};
use env;

macro_rules! try_or_sleep {
    ($expr:expr, $($arg:tt),*) => (match $expr {
        Ok(e) => e,
        Err(ref e) => {
            error!($($arg)*, e);
            thread::sleep(Duration::from_secs(10));
            continue;
        }
    })
}


macro_rules! try_opt {
    ($expr:expr) => (match $expr {
        Some(v) => v,
        None => { return; },
    })
}

pub fn start(ctx: Arc<Context>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let stream = try_or_sleep!(UnixStream::connect(env::docker_host()), "Failed to connect unix socket {}");
            info!("connected to the docker");
            let mut loop_creator = try_or_sleep!(Loop::new(&Config::new()), "Failed while initializing loop {}");
            try_or_sleep!(loop_creator.add_machine_with(move |scope| {
                Docker::new(stream, scope)
            }), "Failed while adding machine {}");
            try_or_sleep!(loop_creator.run(ctx.clone()), "Failed while listening docker events {}");
        }
    })
}

enum Docker {
    Connecting(UnixStream),
    Header(Buf<UnixStream>),
    Stream(Buf<UnixStream>),
}

impl Docker {    
    fn new(stream: UnixStream, scope: &mut EarlyScope) -> Response<Self, <Self as Machine>::Seed> {
        scope.register(&stream, EventSet::writable() | EventSet::readable(), PollOpt::level()).unwrap();
        Response::ok(Docker::Connecting(stream))
    }

    fn request(stream: &mut UnixStream) -> Result<()> {
        try!(write!(stream, "GET /events HTTP/1.1\r\n"));
        try!(write!(stream, "Host: 127.0.0.1\r\n"));
        try!(write!(stream, "\r\n"));
        try!(stream.flush());
        Ok(())
    }

    fn on_event(json: &Json, _scope: &mut Scope<<Self as Machine>::Context>) {
        let status = try_opt!(json.find("status").and_then(Json::as_string));
        if status != "start" && status != "stop" && status != "die" {
            return;
        }
        
    }
}

impl Machine for Docker {
    type Context = Arc<Context>;
    type Seed = Void;

    fn create(seed: Self::Seed, _scope: &mut Scope<Self::Context>) -> Response<Self, Void> {
        unreachable(seed);
    }

    fn ready(self, events: EventSet, scope: &mut Scope<Self::Context>) -> Response<Self,  Self::Seed> {
        match self {
            Docker::Connecting(mut stream) => {
                if !events.is_writable() {
                    return Response::ok(Docker::Connecting(stream));
                }
                Docker::request(&mut stream).unwrap();
                Response::ok(Docker::Header(Buf::new(stream)))
            },
            Docker::Header(mut stream) => {
                if !events.is_readable() {
                    return Response::ok(Docker::Header(stream));
                }
                let parsed = {
                    let mut headers = [httparse::EMPTY_HEADER; 4];
                    let mut response = httparse::Response::new(&mut headers);
                    let buf = stream.fill_buf().unwrap();
                    match response.parse(buf).unwrap() {
                        httparse::Status::Complete(len) => response.code.and_then(move |c| match c {
                            200 => Some(len),
                            _ => None,
                        }),
                        _ => None,
                    }
                };
                    
                if let Some(len) = parsed {
                    info!("Header is responsed with 200");
                    stream.consume(len);
                    Response::ok(Docker::Stream(stream))
                } else {
                    Response::ok(Docker::Header(stream))
                }
            },
            Docker::Stream(mut stream) => {
                if !events.is_readable() {
                    return Response::ok(Docker::Stream(stream));
                }
                stream.fill_buf().unwrap();
                loop {
                    let parsed = httparse::parse_chunk_size(stream.as_slice()).unwrap();
                    if let httparse::Status::Complete((len, size)) = parsed {
                        if size == 0 {
                            stream.consume(len);
                            continue;
                        }
                        let size = size as usize;
                        if stream.pos < len + size {
                            break
                        }
                        let parsed = {
                            let buf = &stream.as_slice()[len..(len+size)];
                            Json::from_str(std::str::from_utf8(buf).unwrap())
                        };
                        stream.consume(len + size);
                        if let Ok(ref json) = parsed {
                            Docker::on_event(json, scope);
                        }
                    } else {
                        break;
                    }
                }
                Response::ok(Docker::Stream(stream))
            },
        }
    }
    
    fn spawned(self, _scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
        unimplemented!();
    }

    fn timeout(self, _scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
        unimplemented!();
    }

    fn wakeup(self, _scope: &mut Scope<Self::Context>) -> Response<Self, Self::Seed> {
        unimplemented!();
    }
}

static DEFAULT_BUF_SIZE: usize = 32 * 1024;

struct Buf<R> {
    reader: R,
    buf: Box<[u8]>,
    pos: usize,
}

impl<R> Buf<R> where R: Read {
    fn new(reader: R) -> Self {
        Buf {
            reader: reader,
            buf: vec![0; DEFAULT_BUF_SIZE].into_boxed_slice(),
            pos: 0,
        }
    }

    fn as_slice(&self) -> &[u8] {
        &(&self.buf)[..self.pos]
    }

    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.pos < self.buf.len() {
            self.pos += try!(self.reader.read(&mut self.buf[self.pos..]));
        }
        Ok(&self.buf[..self.pos])
    }

    fn consume(&mut self, amt: usize) {
        if amt < self.pos {
            self.pos = self.pos - amt;
            unsafe {
                std::ptr::copy(self.buf.as_ptr().offset(amt as isize), self.buf.as_mut_ptr(), self.pos);
            }
        } else {
            self.pos = 0;
        }
    }
}
