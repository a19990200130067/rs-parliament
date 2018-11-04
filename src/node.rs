use std::collections::{ HashMap, HashSet };
use std::vec::Vec;
use std::hash::Hash;
use messaging::*;
use messages::*;
use leader::*;
use replica::*;
use acceptor::*;
use statemachine::*;

use rand::{thread_rng, Rng};

pub trait Node {
    type PollItem;

    fn non_blocking_processing(&mut self) -> Result<(), i32>;
}

pub struct LeaderNode<'a, CmdT, ResultT> {
    server: ZmqServer<'a, Message<CmdT, ResultT>>,
    leader: Leader<'a, CmdT>,
    server_addrs: &'a HashMap<ServerID, Addr>,
}

impl<'a, CmdT, ResultT> LeaderNode<'a, CmdT, ResultT> where
    CmdT: std::fmt::Debug {
    pub fn new(ctx: &'a zmq::Context, addr: &Addr, 
               acceptors: &'a HashSet<ServerID>,
               replica: &'a HashSet<ServerID>,
               my_id: ServerID,
               server_addrs: &'a HashMap<ServerID, Addr>) -> Self {
        LeaderNode {
            server: ZmqServer::bind(ctx, addr),
            leader: Leader::new(acceptors, replica, my_id),
            server_addrs: server_addrs,
        }
    }
}

impl<'a, CmdT, ResultT> Node for LeaderNode<'a, CmdT, ResultT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ResultT: serde::Serialize + serde::de::DeserializeOwned + Clone {
    type PollItem = ();
    
    fn non_blocking_processing(&mut self) -> Result<(), i32> {
        let maybe_msg = self.server.try_recv_timeout(100);
        maybe_msg.map_or(Err(-1), |msg| {
            let to_send = self.leader.handle_msg(&msg);
            to_send.into_iter().map(|(server_id, m)| {
                self.server_addrs.get(&server_id).map(|addr| {
                    let mut c = ZmqClient::connect(addr);
                    c.send(&m);
                });
            }).collect::<()>();
            Ok(())
        })
    }
}


pub struct AcceptorNode<'a, CmdT, ResultT> {
    server: ZmqServer<'a, Message<CmdT, ResultT>>,
    acceptor: Acceptor<CmdT>,
    server_addrs: &'a HashMap<ServerID, Addr>,
}

impl<'a, CmdT, ResultT> AcceptorNode<'a, CmdT, ResultT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ResultT: serde::Serialize + serde::de::DeserializeOwned + Clone {
    pub fn new(ctx: &'a zmq::Context, addr: &Addr,
               my_id: ServerID,
               server_addrs: &'a HashMap<ServerID, Addr>) -> Self {
        AcceptorNode {
            server: ZmqServer::bind(ctx, addr),
            acceptor: Acceptor::new(my_id),
            server_addrs: server_addrs,
        }
    }
}

impl<'a, CmdT, ResultT> Node for AcceptorNode<'a, CmdT, ResultT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ResultT: serde::Serialize + serde::de::DeserializeOwned + Clone {
    type PollItem = ();

    fn non_blocking_processing(&mut self) -> Result<(), i32> {
        let maybe_msg = self.server.try_recv_timeout(100);
        maybe_msg.map_or(Err(-1), |msg| {
            let to_send = self.acceptor.handle_msg::<ResultT>(&msg);
            to_send.into_iter().map(|(server_id, m)| {
                self.server_addrs.get(&server_id).map(|addr| {
                    let mut c = ZmqClient::connect(addr);
                    c.send(&m);
                });
            }).collect::<()>();
            Ok(())
        })
    }
}


pub struct ReplicaNode<'a, S: StateMachine> {
    server: ZmqServer<'a, Message<S::Op, S::Result>>,
    replica: Replica<'a, S>,
    server_addrs: &'a HashMap<ServerID, Addr>,
}

impl<'a, S> ReplicaNode<'a, S> where
    S: StateMachine,
    S::Op: serde::Serialize + serde::de::DeserializeOwned + Clone + Eq + Hash + std::fmt::Debug,
    S::Result: serde::Serialize + serde::de::DeserializeOwned + Clone {
    pub fn new(ctx: &'a zmq::Context, addr: &Addr,
               my_id: ServerID,
               server_addrs: &'a HashMap<ServerID, Addr>,
               leaders: &'a HashSet<ServerID>) -> Self {
        ReplicaNode {
            server: ZmqServer::bind(ctx, addr),
            replica: Replica::new(leaders),
            server_addrs: server_addrs,
        }
    }
}

impl<'a, S> Node for ReplicaNode<'a, S> where
    S: StateMachine,
    S::Op: serde::Serialize + serde::de::DeserializeOwned + Clone + Eq + Hash + std::fmt::Debug,
    S::Result: serde::Serialize + serde::de::DeserializeOwned + Clone {
    type PollItem = ();

    fn non_blocking_processing(&mut self) -> Result<(), i32> {
        let maybe_msg = self.server.try_recv_timeout(100);
        maybe_msg.map_or(Err(-1), |msg| {
            let (to_send_server, to_send_client) = self.replica.handle_msg(&msg);
            to_send_server.into_iter().map(|(server_id, m)| {
                self.server_addrs.get(&server_id).map(|addr| {
                    let mut c = ZmqClient::connect(addr);
                    c.send(&m);
                });
            }).collect::<()>();
            to_send_client.into_iter().map(|(addr, m)| {
                let mut c = ZmqClient::connect(&addr);
                c.send(&m);
            }).collect::<()>();
            Ok(())
        })
    }
}


pub struct ClientNode<'a, S: StateMachine> {
    server: ZmqServer<'a, Message<S::Op, S::Result>>,
    replicas: &'a HashSet<Addr>,
}

impl<'a, S> ClientNode<'a, S> where
    S: StateMachine,
    S::Op: serde::Serialize + serde::de::DeserializeOwned,
    S::Result: serde::Serialize + serde::de::DeserializeOwned {
    pub fn new(ctx: &'a zmq::Context, addr: &Addr,
               replicas: &'a HashSet<Addr>) -> Self {
        ClientNode {
            server: ZmqServer::bind(ctx, addr),
            replicas: replicas,
        }
    }

    pub fn send_cmd(&mut self, op: &Message<S::Op, S::Result>) -> Result<S::Result, i32> {
        let addr_vec = self.replicas.iter()
            .map(|a| a.clone())
            .collect::<Vec<_>>();
        let maybe_addr = thread_rng().choose(addr_vec.as_slice());
        maybe_addr.map_or(Err(-1), |addr| {
            let mut c = ZmqClient::connect(addr);
            c.send(op);
            self.server.try_recv_timeout(1000).map_or(Err(-2), |msg| {
                match msg {
                    Message::Response { cid: _, result } => Ok(result),
                    _ => Err(-3),
                }
            })
        })
    }
}
