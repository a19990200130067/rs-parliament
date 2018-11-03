#[macro_use]
extern crate serde_derive;
extern crate rs_parliament;
use rs_parliament::node::*;
use rs_parliament::lockmachine::*;
use rs_parliament::messaging::*;
use rs_parliament::messages::*;
extern crate clap;
use clap::{ App, Arg };
use std::collections::{ HashMap, HashSet };

fn main() {
    let matches = App::new("lock_leader")
        .version("1.0")
        .arg(Arg::with_name("PORT_NUMBER")
             .required(true)
             .index(1))
        .arg(Arg::with_name("MY_IDX")
             .required(true)
             .index(2))
        .get_matches();

    let lh = "127.0.0.1";
    let server_addr_vec = vec![(0, Addr::new(lh, 8000))];

    let server_addrs: HashMap<_, _> = server_addr_vec.into_iter().collect();
    let leaders: HashSet<_> = vec![0].into_iter().collect();

    let ctx = zmq::Context::new();
    let port = matches.value_of("PORT_NUMER").unwrap();
    let addr = Addr { addr: "127.0.0.1".to_string(), port: port.to_string().parse::<u16>().unwrap() };
    let idx = matches.value_of("MY_IDX").unwrap().to_string().parse::<ServerID>().unwrap();

    let _replica = ReplicaNode::<LockMachine>::new(&ctx, &addr, idx, &server_addrs, &leaders);
}
