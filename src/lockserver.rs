#[macro_use]
extern crate serde_derive;
extern crate rs_parliament;
use rs_parliament::messaging::*;
extern crate zmq;
use std::thread;
use std::time::Duration;


#[derive(Serialize, Deserialize)]
struct Foo {
    a: u32,
}

fn main() {
    println!("Starting lock server");
    let ctx = zmq::Context::new();
    
    let mut socket = ctx.socket(zmq::ROUTER).unwrap();
    assert!(socket.bind("tcp://*:8000").is_ok());
    let mut poll_list = vec!(socket.as_poll_item(zmq::POLLIN));
    loop {
        let poll_result = zmq::poll(poll_list.as_mut_slice(), 100);
        if poll_result.ok().map_or(false, |s| s > 0) {
            let msg = socket.recv_multipart(zmq::DONTWAIT);
            if true {
                println!("got message {:?}", msg);
                println!("num parts {}", msg.ok().map_or(0, (|v| v.len())));
            }
        }
    }
    //socket.connect("tcp://127.0.0.1:1234").unwrap();
    //socket.send_str("hello world!", 0).unwrap();
    let _addr: Addr = Addr { addr: "127.0.0.1".to_string(), port: 8080 };
}
