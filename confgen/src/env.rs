use std::env;
use std::str::FromStr;
use std::ops::Deref;
use std::path::{Path,PathBuf};

fn var<T>(name: &str) -> Option<T> where T: FromStr {
    env::var(name).ok().and_then(|v| T::from_str(&v).ok())
}

pub fn dest() -> &'static Path {
    lazy_static! {
        static ref DEST: PathStore = var("WEB_PROXY_DEST").unwrap_or(PathStore::new("/etc/nginx/conf.d/default.conf"));
    }
    &&*DEST
}

pub fn docker_host() -> &'static Path {
    lazy_static! {
        static ref DOCKER_HOST: PathStore = var("DOCKER_HOST").unwrap_or(PathStore::new("/var/run/docker.sock"));
    }
    &&*DOCKER_HOST
}

struct PathStore(PathBuf);
impl PathStore {
    fn new(s: &str) -> PathStore {
        PathStore(PathBuf::from(s))
    }
}
impl FromStr for PathStore {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PathStore(PathBuf::from(s)))
    }
}
impl Deref for PathStore {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::var;
    
    #[test]
    fn var_dest_default_test() {
        let key = "WEB_PROXY_DEST";
        env::remove_var(key);
        let actual: Option<String> = var(key);
        assert_eq!(actual, None);
    }

    #[test]
    fn var_dest_env_test() {
        let key = "WEB_PROXY_DEST";
        let val = "/etc/nginx/conf.d/test.conf";
        env::set_var(key, val);
        let actual: Option<String> = var(key);
        assert_eq!(actual, Some(val.to_string()));
    }
}

