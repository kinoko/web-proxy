
use std;
use std::io::{Write, Read};
use std::time::Duration;
use std::thread;
use std::marker::PhantomData;
use thread_scoped::{scoped, JoinGuard};
use rotor::mio::unix::UnixStream;
use rotor::{Machine, Response, Scope, EarlyScope, EventSet, Loop, Config, PollOpt};
use rotor::void::{Void, unreachable};
use httparse;
use serialize::json::Json;

use dispatcher::{Context, Result};
use env;

pub fn start<'a>(ctx: &'a Context) -> JoinGuard<'a, ()> {
    let main = move || {
        loop {
            let stream = try_or_sleep!(UnixStream::connect(env::docker_host()), "Failed to connect unix socket {}");
            info!("connected to the docker");
            let mut loop_creator = try_or_sleep!(Loop::new(&Config::new()), "Failed while initializing loop {}");
            try_or_sleep!(loop_creator.add_machine_with(move |scope| {
                Docker::new(stream, scope)
            }), "Failed while adding machine {}");
            try_or_sleep!(loop_creator.run(ctx), "Failed while listening docker events {}");
        }
    };
    unsafe { scoped(main) }
}



enum Docker<'a> {
    Connecting(DockerStream<'a>),
    Header(DockerBufStream<'a>),
    Stream(DockerBufStream<'a>),
}
struct DockerStream<'a> {
    stream: UnixStream,
    phantom: PhantomData<&'a u8>,
}
struct DockerBufStream<'a> {
    stream: Buf<UnixStream>,
    phantom: PhantomData<&'a u8>,
}

impl<'a> DockerStream<'a> {
    fn new(stream: UnixStream) -> Self {
        DockerStream {
            stream: stream,
            phantom: PhantomData,
        }
    }
}
impl<'a> DockerBufStream<'a> {
    fn new(stream: Buf<UnixStream>) -> Self {
        DockerBufStream {
            stream: stream,
            phantom: PhantomData
        }
    }
}

use docker::Docker::*;

impl<'a> Docker<'a> {
    fn new(stream: UnixStream, scope: &mut EarlyScope) -> Response<Self, <Self as Machine>::Seed> {
        try_rotor!(scope.register(&stream, EventSet::writable(), PollOpt::level()));
        Response::ok(Connecting(DockerStream::new(stream)))
    }

    fn request(stream: &mut UnixStream) -> Result<()> {
        try!(write!(stream, "GET /events HTTP/1.1\r\n"));
        try!(write!(stream, "Host: 127.0.0.1\r\n"));
        try!(write!(stream, "\r\n"));
        try!(stream.flush());
        Ok(())
    }

    fn respond_with_ok(stream: &mut Buf<UnixStream>) -> Result<Option<usize>> {
        let mut headers = [httparse::EMPTY_HEADER; 4];
        let mut response = httparse::Response::new(&mut headers);
        let buf = try!(stream.fill_buf());
        if let httparse::Status::Complete(len) = try!(response.parse(buf)) {
            if response.code == Some(200) {
                return Ok(Some(len));
            }
        }
        Ok(None)
    }

    fn on_connecting(mut stream: UnixStream, scope: &mut Scope<<Self as Machine>::Context>) -> Response<Self, <Self as Machine>::Seed> {
        try_rotor!(Docker::request(&mut stream));
        try_rotor!(scope.reregister(&stream, EventSet::readable(), PollOpt::level()));
        Response::ok(Header(DockerBufStream::new(Buf::new(stream))))
    }

    fn on_header(mut stream: Buf<UnixStream>) -> Response<Self, <Self as Machine>::Seed> {
        if let Some(len) = try_rotor!(Docker::respond_with_ok(&mut stream)) {
            info!("Header is responsed with 200");
            stream.consume(len);
            Response::ok(Stream(DockerBufStream::new(stream)))
        } else {
            Response::ok(Header(DockerBufStream::new(stream)))
        }                    
    }

    fn chunk(stream: &mut Buf<UnixStream>) -> Result<(usize, Option<Json>)> {
        if let httparse::Status::Complete((len, size)) = try!(httparse::parse_chunk_size(stream.as_slice())) {
            let size = size as usize;
            if stream.pos >= len + size {
                let mut len = len;
                let mut json = None;
                if size > 0 {
                    let buf = try!(std::str::from_utf8(&stream.as_slice()[len..(len+size)]));
                    len += size;
                    json = Some(try!(Json::from_str(buf)));
                }
                return Ok((len, json));
            }
        }
        Ok((0, None))
    }

    fn on_event(json: &Json, scope: &mut Scope<<Self as Machine>::Context>) {
        let status = try_opt!(json.find("status").and_then(Json::as_string));
        if status != "start" && status != "stop" && status != "die" {
            return;
        }
        info!("Container {}", status);
        let mut wait = scope.changed.lock().unwrap();
        *wait = *wait || true;
        scope.lock.notify_all();
    }

    fn on_stream(mut stream: Buf<UnixStream>, scope: &mut Scope<<Self as Machine>::Context>) -> Response<Self, <Self as Machine>::Seed> {
        try_rotor!(stream.fill_buf());
        loop {
            match try_rotor!(Docker::chunk(&mut stream)) {
                (0, None) => { break; },
                (len, None) => { stream.consume(len); },
                (len, Some(ref json)) => {
                    stream.consume(len);
                    Docker::on_event(json, scope);
                }
            }
        }
        Response::ok(Stream(DockerBufStream::new(stream)))
    }
}

impl<'a> Machine for Docker<'a> {
    type Context = &'a Context;
    type Seed = Void;

    fn create(seed: Self::Seed, _scope: &mut Scope<Self::Context>) -> Response<Self, Void> {
        unreachable(seed);
    }

    fn ready(self, _events: EventSet, scope: &mut Scope<Self::Context>) -> Response<Self,  Self::Seed> {
        match self {
            Docker::Connecting(d) => Docker::on_connecting(d.stream, scope),
            Docker::Header(d) => Docker::on_header(d.stream),
            Docker::Stream(d) => Docker::on_stream(d.stream, scope),
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
