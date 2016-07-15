
use std::vec::Vec;
use std::collections::{HashMap};
use std::net::{IpAddr};
use std::fs::File;
use std::io::{Write, Read, Seek, SeekFrom};
use tempfile;

pub struct Config<'a> {
    pub hosts: HashMap<&'a str, VirtualHost<'a>>,
    pub containers: Vec<Container>,
}

pub struct Location<'a> {
    pub container: &'a Container,
    pub prefix: String,
}

pub struct VirtualHost<'a> {
    pub name: String,
    pub locations: Vec<Location<'a>>,
}

pub struct Container {
    pub name: String,
    pub address: IpAddr,
    pub port: u16,
}


impl<'a> Config<'a> {
    pub fn new() -> Self {
        Config {
            hosts: HashMap::new(),
            containers: Vec::new(),
        }
    }

    pub fn generate(&self) {
        let mut tmp: File = tempfile::tempfile().unwrap();
        
        tmp.seek(SeekFrom::Start(0)).unwrap();
    }
}
