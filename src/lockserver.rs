extern crate rs_parliament;
use rs_parliament::node::*;
use rs_parliament::lockmachine::*;
use rs_parliament::messaging::*;
use rs_parliament::messages::*;
extern crate clap;
use clap::{ App, Arg, SubCommand, AppSettings, ArgMatches };
use std::collections::{ HashMap, HashSet };

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

    let replicas: HashMap<_, _> = replica_vec.iter().map(|(i, a)| (i.clone(), a.clone())).collect();
    let leaders: HashMap<_, _> = leader_vec.iter().map(|(i, a)| (i.clone(), a.clone())).collect();
    let acceptors: HashMap<_, _> = acceptor_vec.iter().map(|(i, a)| (i.clone(), a.clone())).collect();
    
    let mut server_addrs = replicas.clone();
    leaders.iter().map(|(k, v)| { server_addrs.insert(*k, v.clone()); }).collect::<()>();
    acceptors.iter().map(|(k, v)| { server_addrs.insert(*k, v.clone()); }).collect::<()>();

    let replicas: HashSet<_> = replicas.into_iter().map(|(k, _v)| k).collect();
    let leaders: HashSet<_> = leaders.into_iter().map(|(k, _v)| k).collect();
    let acceptors: HashSet<_> = acceptors.into_iter().map(|(k, _v)| k).collect();

    let idx_arg = Arg::with_name("IDX")
        .required(true)
        .index(1);
    let matches = App::new("lock_leader")
        .version("1.0")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("replica")
                    .arg(idx_arg.clone()))
        .subcommand(SubCommand::with_name("leader")
                    .arg(idx_arg.clone()))
        .subcommand(SubCommand::with_name("acceptor")
                    .arg(idx_arg.clone()))
        .get_matches();

    let get_addr_idx = |m: &ArgMatches, v: &Vec<(ServerID, Addr)>| -> (Addr, ServerID) {
        let i = m.value_of("IDX").expect("parse idx").to_string().parse::<usize>().unwrap();
        let (idx, addr) = v.get(i).expect("unknown server index");
        (addr.clone(), *idx)
    };
    matches.subcommand_matches("leader").map(|matches| {
        let (addr, idx) = get_addr_idx(matches, &leader_vec);
        println!("leader addr: {:?}, idx: {:?}, pid: {}", addr, idx, std::process::id());
        let mut node = LeaderNode::<LockOp, LockResult, 
                                    UdpRecver<_>, UdpSender<_>>::new(&addr, 
                                                                     &acceptors, &replicas, idx, 
                                                                     &server_addrs);
        loop {
            let _ = node.non_blocking_processing();
        }
    });
    
    matches.subcommand_matches("acceptor").map(|matches| {
        let (addr, idx) = get_addr_idx(matches, &acceptor_vec);
        println!("acceptor addr: {:?}, idx: {:?}, pid: {}", addr, idx, std::process::id());
        let mut node = AcceptorNode::<LockOp, LockResult, 
                                      UdpRecver<_>, UdpSender<_>>::new(&addr, 
                                                                       idx, &server_addrs);
        loop {
            let _ = node.non_blocking_processing();
        }
    });

    matches.subcommand_matches("replica").map(|matches| {
        let (addr, idx) = get_addr_idx(matches, &replica_vec);
        println!("replica addr: {:?}, idx: {:?}, pid: {}", addr, idx, std::process::id());
        let mut node = ReplicaNode::<LockMachine, 
                                     UdpRecver<_>, UdpSender<_>>::new(&addr, idx, &server_addrs, &leaders);
        loop {
            let _ = node.non_blocking_processing();
        }
    });
}
