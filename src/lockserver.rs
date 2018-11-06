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

    let replica_map: HashMap<_, _> = replica_vec.iter().map(|(i, a)| (i.clone(), a.clone())).collect();
    let leader_map: HashMap<_, _> = leader_vec.iter().map(|(i, a)| (i.clone(), a.clone())).collect();
    let acceptor_map: HashMap<_, _> = acceptor_vec.iter().map(|(i, a)| (i.clone(), a.clone())).collect();
    
    let mut server_addrs = replica_map.clone();
    leader_map.iter().map(|(k, v)| { server_addrs.insert(*k, v.clone()); }).collect::<()>();
    acceptor_map.iter().map(|(k, v)| { server_addrs.insert(*k, v.clone()); }).collect::<()>();

    let replicas: HashSet<_> = replica_map.into_iter().map(|(k, _v)| k).collect();
    let leaders: HashSet<_> = leader_map.into_iter().map(|(k, _v)| k).collect();
    let acceptors: HashSet<_> = acceptor_map.into_iter().map(|(k, _v)| k).collect();

    let idx_arg = Arg::with_name("IDX")
        .required(true)
        .index(1);
    let client_args = vec![Arg::with_name("lockid").required(true).index(1),
                           Arg::with_name("clientid").required(false).index(2)];
                           
    let matches = App::new("lock_leader")
        .version("1.0")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("replica")
                    .arg(idx_arg.clone()))
        .subcommand(SubCommand::with_name("leader")
                    .arg(idx_arg.clone()))
        .subcommand(SubCommand::with_name("acceptor")
                    .arg(idx_arg.clone()))
        .subcommand(SubCommand::with_name("client")
                    .setting(AppSettings::SubcommandRequired)
                    .arg(Arg::with_name("port").required(true))
                    .subcommand(SubCommand::with_name("lock")
                                .args(client_args.as_slice()))
                    .subcommand(SubCommand::with_name("unlock")
                                .args(client_args.as_slice())))
        .get_matches();

    let get_addr_idx = |m: &ArgMatches, v: &Vec<(ServerID, Addr)>| -> (Addr, ServerID) {
        let i = m.value_of("IDX").expect("parse idx").to_string().parse::<usize>().unwrap();
        let (idx, addr) = v.get(i).expect("unknown server index");
        (addr.clone(), *idx)
    };

    //type MsgT = Message<LockOp, LockResult>;
    // type ServerT<T> = UdpRecver<T>;
    // type ClientT<T> = UdpSender<T>;
    type ServerT<T> = ZmqServer<T>;
    type ClientT<T> = ZmqClient<T>;

    matches.subcommand_matches("leader").map(|matches| {
        let (addr, idx) = get_addr_idx(matches, &leader_vec);
        println!("leader addr: {:?}, idx: {:?}, pid: {}", addr, idx, std::process::id());
        let mut node = LeaderNode::<LockOp, LockResult, 
                                    ServerT<_>, ClientT<_>>::new(&addr, 
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
                                      ServerT<_>, ClientT<_>>::new(&addr, 
                                                                       idx, &server_addrs);
        loop {
            let _ = node.non_blocking_processing();
        }
    });

    matches.subcommand_matches("replica").map(|matches| {
        let (addr, idx) = get_addr_idx(matches, &replica_vec);
        println!("replica addr: {:?}, idx: {:?}, pid: {}", addr, idx, std::process::id());
        let mut node = ReplicaNode::<LockMachine, 
                                     ServerT<_>, ClientT<_>>::new(&addr, idx, &server_addrs, &leaders);
        loop {
            let _ = node.non_blocking_processing();
        }
    });

    matches.subcommand_matches("client").map(|matches| {
        let port_num = matches.value_of("port")
            .and_then(|s| s.to_string().parse::<u16>().ok())
            .expect("port number");

        let addr = Addr { addr: "127.0.0.1".to_string(), port: port_num };
        
        let extract_arg = |m: &ArgMatches| -> (u64, u64) {
            let lockid = m.value_of("lockid")
                .and_then(|s| s.to_string().parse::<u64>().ok())
                .unwrap_or(0);
            let clientid = m.value_of("clientid")
                .and_then(|s| s.to_string().parse::<u64>().ok())
                .unwrap_or(port_num as u64);
            (lockid, clientid)
        };
        
        let replicas: HashSet<_> = replica_vec.iter().map(|(_, a)| a.clone()).collect();

        let mut client = ClientNode::<LockMachine, 
                                      ServerT<_>, ClientT<_>>::new(&addr, &replicas);
        
        matches.subcommand_matches("lock").map(|matches| {
            let (lockid, clientid) = extract_arg(matches);
            let cmd = LockOp::TryLock(lockid, clientid);
            let req = Message::Request { cid: addr.clone(), cmd: cmd };
            client.send_cmd(&req).ok().map(|r| {
                println!("result: {:?}", r);
            });
        });
        matches.subcommand_matches("unlock").map(|matches| {
            let (lockid, clientid) = extract_arg(matches);
            let cmd = LockOp::TryUnlock(lockid, clientid);
            let req = Message::Request { cid: addr.clone(), cmd: cmd };
            client.send_cmd(&req).ok().map(|r| {
                println!("result: {:?}", r);
            });
        });
    });
}
