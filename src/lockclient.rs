extern crate rs_parliament;
use rs_parliament::node::*;
use rs_parliament::lockmachine::*;
use rs_parliament::messaging::*;
use rs_parliament::messages::*;
extern crate clap;
use clap::{ App, Arg, SubCommand, AppSettings };
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

    let replicas: HashMap<_, _> = replica_vec.into_iter().collect();
    let leaders: HashMap<_, _> = leader_vec.into_iter().collect();
    let acceptors: HashMap<_, _> = acceptor_vec.into_iter().collect();
    
    let mut server_addrs = replicas.clone();
    leaders.iter().map(|(k, v)| { server_addrs.insert(*k, v.clone()); }).collect::<()>();
    acceptors.iter().map(|(k, v)| { server_addrs.insert(*k, v.clone()); }).collect::<()>();

    let replicas: HashSet<_> = replicas.into_iter().map(|(_k, v)| v).collect();
    //let leaders: HashSet<_> = leaders.into_iter().map(|(k, _v)| k).collect();
    //let acceptors: HashSet<_> = acceptors.into_iter().map(|(k, _v)| k).collect();

    let matches = App::new("lock_client")
        .version("1.0")
        .setting(AppSettings::SubcommandRequired)
        .arg(Arg::with_name("PORT")
             .required(true)
             .index(1))
        .subcommand(SubCommand::with_name("lock")
                    .arg(Arg::with_name("LOCKID")
                         .required(true).index(1)))
        .subcommand(SubCommand::with_name("unlock")
                    .arg(Arg::with_name("LOCKID")
                         .required(true).index(1)))
        .get_matches();


    let port = matches.value_of("PORT").expect("port num");
    let addr = Addr { addr: "127.0.0.1".to_string(), port: port.to_string().parse::<u16>().expect("parse port") };

    println!("addr: {:?}, pid: {}", addr, std::process::id());

    let mut client = ClientNode::<LockMachine, 
                                  UdpRecver<_>, UdpSender<_>>::new(&addr, &replicas);
    
    matches.subcommand_matches("lock").map(|m| {
        let lockid = m.value_of("LOCKID").expect("lock id arg").parse().expect("lock id parse");
        let cmd = LockOp::TryLock(lockid, 0);
        let req = Message::Request { cid: addr.clone(), cmd: cmd };
        client.send_cmd(&req).ok().map(|r| {
            println!("result: {:?}", r);
        });
    });

    matches.subcommand_matches("unlock").map(|m| {
        let lockid = m.value_of("LOCKID").expect("lock id arg").parse().expect("lock id parse");
        let cmd = LockOp::TryUnlock(lockid, 0);
        let req = Message::Request { cid: addr.clone(), cmd: cmd };
        client.send_cmd(&req).ok().map(|r| {
            println!("result: {:?}", r);
        });
    });
}
