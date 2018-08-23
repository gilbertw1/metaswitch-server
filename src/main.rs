#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate tokio_core;
extern crate tokio;
extern crate hyper;
extern crate reqwest;
extern crate strsim;
extern crate serde_json;
extern crate regex;
extern crate url;

pub mod service;
pub mod metacritic;

use std::net::IpAddr;
use service::LookupService;

fn main() {
    println!("Creating metacritic lookup handler");
    let handler = metacritic::create_lookup_handler();

    println!("Starting lookup service");
    LookupService::start("0.0.0.0".parse::<IpAddr>().unwrap(), 9000, handler);
}
