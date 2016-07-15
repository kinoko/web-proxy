
use thread_scoped::{scoped, JoinGuard};
use dispatcher::{Context};
use config::Config;

pub fn start<'a>(ctx: &'a Context) -> JoinGuard<'a, ()> {
    let main = move || { inner(ctx) };
    unsafe { scoped(main) }
}

fn inner<'a>(ctx: &'a Context) {
    loop {
        dump();
        let mut wait = ctx.changed.lock().unwrap();
        while !*wait {
            wait = ctx.lock.wait(wait).unwrap();
        }
        *wait = false;
    }
}

fn dump() {
    info!("Dumping");
    let mut config = Config::new();

    config.generate();
}
