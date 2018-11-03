extern crate rs_parliament;
use rs_parliament::node::*;
use rs_parliament::lockmachine::*;
use rs_parliament::messaging::*;
use rs_parliament::messages::*;
extern crate clap;
use clap::{ App, Arg };
use std::collections::{ HashMap };

fn main() {
    let lh = "127.0.0.1";

    // replica
    let replica_vec = vec![(0, Addr::new(lh, 8000)),
                           (1, Addr::new(lh, 8001))];
    // leader
    let leader_vec = vec![(10, Addr::new(lh, 9001)),
                          (11, Addr::new(lh, 9002))];
    // acceptor
    let acceptor_vec = vec![(20, Addr::new(lh, 9101)),
                            (21, Addr::new(lh, 9102)),
                            (22, Addr::new(lh, 9103))];

    let replicas: HashMap<_, _> = replica_vec.into_iter().collect();
    let leaders: HashMap<_, _> = leader_vec.into_iter().collect();
    let acceptors: HashMap<_, _> = acceptor_vec.into_iter().collect();
    
    let mut server_addrs = replicas.clone();
    leaders.iter().map(|(k, v)| { server_addrs.insert(*k, v.clone()); }).collect::<()>();
    acceptors.iter().map(|(k, v)| { server_addrs.insert(*k, v.clone()); }).collect::<()>();

    //let replicas: HashSet<_> = replicas.into_iter().map(|(k, _v)| k).collect();
    //let leaders: HashSet<_> = leaders.into_iter().map(|(k, _v)| k).collect();
    //let acceptors: HashSet<_> = acceptors.into_iter().map(|(k, _v)| k).collect();

    let matches = App::new("lock_acceptor")
        .version("1.0")
        .arg(Arg::with_name("PORT")
             .required(true)
             .index(1))
        .arg(Arg::with_name("IDX")
             .required(true)
             .index(2))
        .get_matches();


    let ctx = zmq::Context::new();
    let port = matches.value_of("PORT").expect("port num");
    let addr = Addr { addr: "127.0.0.1".to_string(), port: port.to_string().parse::<u16>().expect("parse port") };
    let idx = matches.value_of("IDX").expect("parse idx").to_string().parse::<ServerID>().unwrap();

    println!("addr: {:?}, idx: {:?}", addr, idx);

    let mut acceptor = AcceptorNode::<LockOp, LockResult>::new(&ctx, &addr, 
                                                               idx, &server_addrs);
    loop {
        let _ = acceptor.non_blocking_processing();
    }
}
