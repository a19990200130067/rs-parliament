#[macro_use]
extern crate serde_derive;
extern crate rs_parliament;
use rs_parliament::messaging::*;

#[derive(Serialize, Deserialize)]
struct Foo {
    a: u32,
}

fn main() {
    println!("Starting lock server");
    let addr: Addr = Addr { addr: "127.0.0.1", port: 8080 };
    let mut recv = UdpRecver::<Foo>::bind(addr);
    loop {
        recv.try_recv().map(|msg| {
            println!("got: {}", msg.a);
        });
    }
}
