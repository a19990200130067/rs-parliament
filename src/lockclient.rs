#[macro_use]
extern crate serde_derive;
extern crate rs_parliament;
use rs_parliament::messaging::*;
use std::{thread, time};

#[derive(Serialize, Deserialize)]
struct Foo {
    a: u32,
}

fn main() {
    println!("Starting lock client");
    let addr: Addr = Addr { addr: "127.0.0.1", port: 8080 };
    let mut send = UdpSender::<Foo>::connect(addr);
    thread::sleep(time::Duration::from_millis(100));
    println!("{:?}", send.send(&Foo { a: 90 }));
}
