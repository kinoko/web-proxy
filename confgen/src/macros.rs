

macro_rules! try_or_exit {
    ($expr:expr, $($arg:tt)*) => (match $expr {
        Ok(e) => e,
        Err(ref e) => {
            error!($($arg)*, e);
            exit(1);
        }
    })
}


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

macro_rules! try_rotor {
    ($expr:expr) => (match $expr {
        Ok(v) => v,
        Err(e) => { return ::rotor::Response::error(std::convert::From::from(e)); }
    })
}
