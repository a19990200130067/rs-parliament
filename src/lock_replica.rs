extern crate rs_parliament;
use rs_parliament::node::*;
use rs_parliament::lockmachine::*;
use rs_parliament::messaging::*;
use rs_parliament::messages::*;
extern crate clap;
use clap::{ App, Arg };
use std::collections::{ HashMap, HashSet };

fn main() {
    let lh = "127.0.0.1";
    let server_addr_vec = vec![
        // replica
        (0, Addr::new(lh, 8000)),
        (1, Addr::new(lh, 8001)),
        // leader
        (10, Addr::new(lh, 9001)),
        (11, Addr::new(lh, 9002)),
        // acceptor
        (20, Addr::new(lh, 9101)),
        (21, Addr::new(lh, 9102)),
        (22, Addr::new(lh, 9103))];

    let server_addrs: HashMap<_, _> = server_addr_vec.into_iter().collect();
    let leaders: HashSet<_> = vec![10, 11].into_iter().collect();

    let matches = App::new("lock_replica")
        .version("1.0")
        .arg(Arg::with_name("PORT")
             .required(true)
             .index(1))
        .arg(Arg::with_name("IDX")
             .required(true)
             .index(2))
        .get_matches();


    let port = matches.value_of("PORT").expect("port num");
    let addr = Addr { addr: "127.0.0.1".to_string(), port: port.to_string().parse::<u16>().expect("parse port") };
    let idx = matches.value_of("IDX").expect("parse idx").to_string().parse::<ServerID>().unwrap();

    println!("replica addr: {:?}, idx: {:?}, pid: {}", addr, idx, std::process::id());

    let mut replica = ReplicaNode::<LockMachine, 
                                    UdpRecver<_>, UdpSender<_>>::new(&addr, idx, &server_addrs, &leaders);
    loop {
        let _ = replica.non_blocking_processing();
    }
}
