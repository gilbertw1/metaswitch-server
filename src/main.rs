#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate hyper;
extern crate reqwest;
extern crate strsim;
extern crate serde_json;
extern crate regex;
extern crate url;
extern crate tokio;

pub mod service;
pub mod metacritic;

use tokio::runtime::Builder;
use std::net::IpAddr;

fn main() {
    println!("Creating metacritic lookup handler");
    let handler = metacritic::create_lookup_handler();

    println!("Starting lookup service");
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(service::start("0.0.0.0".parse::<IpAddr>().unwrap(), 80, handler));
}
