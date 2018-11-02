#[macro_use]
extern crate serde_derive;
extern crate rs_parliament;
use rs_parliament::messaging::*;
use std::{thread, time};
extern crate zmq;
extern crate rand;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[derive(Serialize, Deserialize)]
struct Foo {
    a: u32,
}

fn main() {
    println!("Starting lock client");
    let ctx = zmq::Context::new();
    let _addr: Addr = Addr { addr: "127.0.0.1".to_string(), port: 8080 };
    let mut socket = ctx.socket(zmq::REQ).unwrap();
    socket.connect("tcp://127.0.0.1:8000").unwrap();
    let s = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(300)
        .collect::<String>();
    socket.send(s.as_bytes(), zmq::DONTWAIT).unwrap();
}
