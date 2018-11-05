use std::collections::{ HashMap, HashSet };
use std::vec::Vec;
use std::hash::Hash;
use messaging::*;
use messages::*;
use leader::*;
use replica::*;
use acceptor::*;
use statemachine::*;
use std::marker::PhantomData;

use rand::{thread_rng, Rng};

pub trait Node {
    type PollItem;

    fn non_blocking_processing(&mut self) -> Result<(), i32>;
}

pub struct LeaderNode<'a, CmdT, ResultT, ServerT, ClientT> {
    server: ServerT,
    leader: Leader<'a, CmdT>,
    server_addrs: &'a HashMap<ServerID, Addr>,
    result_type: PhantomData<ResultT>,
    client_type: PhantomData<ClientT>,
}

impl<'a, CmdT, ResultT, ServerT, ClientT> LeaderNode<'a, CmdT, ResultT, ServerT, ClientT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ResultT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ServerT: MsgRecver<Message<CmdT, ResultT>>,
    ClientT: MsgSender<Message<CmdT, ResultT>> {
    pub fn new(addr: &Addr, 
               acceptors: &'a HashSet<ServerID>,
               replica: &'a HashSet<ServerID>,
               my_id: ServerID,
               server_addrs: &'a HashMap<ServerID, Addr>) -> Self {
        LeaderNode {
            server: ServerT::bind(addr),
            leader: Leader::new(acceptors, replica, my_id),
            server_addrs: server_addrs,
            result_type: PhantomData,
            client_type: PhantomData,
        }
    }
}

impl<'a, CmdT, ResultT, ServerT, ClientT> Node for LeaderNode<'a, CmdT, ResultT, ServerT, ClientT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ResultT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ServerT: MsgRecver<Message<CmdT, ResultT>>,
    ClientT: MsgSender<Message<CmdT, ResultT>> {
    type PollItem = ();
    
    fn non_blocking_processing(&mut self) -> Result<(), i32> {
        let maybe_msg = self.server.try_recv_timeout(100);
        let msg = maybe_msg.unwrap_or(Message::Tick);
        let to_send = self.leader.handle_msg(&msg);
        to_send.into_iter().map(|(server_id, m)| {
            //println!("{} sending to {} {:?}", std::process::id(), server_id, m);
            self.server_addrs.get(&server_id).map(|addr| {
                let mut c = ClientT::connect(addr);
                c.send(&m);
            });
        }).collect::<()>();
        Ok(())
    }
}


pub struct AcceptorNode<'a, CmdT, ResultT, ServerT, ClientT> {
    server: ServerT,
    acceptor: Acceptor<CmdT>,
    server_addrs: &'a HashMap<ServerID, Addr>,
    result_type: PhantomData<ResultT>,
    client_type: PhantomData<ClientT>,
}

impl<'a, CmdT, ResultT, ServerT, ClientT> AcceptorNode<'a, CmdT, ResultT, ServerT, ClientT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ResultT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ServerT: MsgRecver<Message<CmdT, ResultT>>,
    ClientT: MsgSender<Message<CmdT, ResultT>> {
    pub fn new(addr: &Addr,
               my_id: ServerID,
               server_addrs: &'a HashMap<ServerID, Addr>) -> Self {
        AcceptorNode {
            server: ServerT::bind(addr),
            acceptor: Acceptor::new(my_id),
            server_addrs: server_addrs,
            result_type: PhantomData,
            client_type: PhantomData,
        }
    }
}

impl<'a, CmdT, ResultT, ServerT, ClientT> Node for AcceptorNode<'a, CmdT, ResultT, ServerT, ClientT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ResultT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ServerT: MsgRecver<Message<CmdT, ResultT>>,
    ClientT: MsgSender<Message<CmdT, ResultT>> {
    type PollItem = ();

    fn non_blocking_processing(&mut self) -> Result<(), i32> {
        let maybe_msg = self.server.try_recv_timeout(100);
        maybe_msg.map_or(Err(-1), |msg| {
            let to_send = self.acceptor.handle_msg::<ResultT>(&msg);
            to_send.into_iter().map(|(server_id, m)| {
                self.server_addrs.get(&server_id).map(|addr| {
                    let mut c = ClientT::connect(addr);
                    c.send(&m);
                });
            }).collect::<()>();
            Ok(())
        })
    }
}


pub struct ReplicaNode<'a, S: StateMachine, ServerT, ClientT> {
    server: ServerT,
    replica: Replica<'a, S>,
    server_addrs: &'a HashMap<ServerID, Addr>,
    client_type: PhantomData<ClientT>,
}

impl<'a, S, ServerT, ClientT> ReplicaNode<'a, S, ServerT, ClientT> where
    S: StateMachine,
    S::Op: serde::Serialize + serde::de::DeserializeOwned + Clone + Eq + Hash + std::fmt::Debug,
    S::Result: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ServerT: MsgRecver<Message<S::Op, S::Result>>,
    ClientT: MsgSender<Message<S::Op, S::Result>> {
    pub fn new(addr: &Addr,
               my_id: ServerID,
               server_addrs: &'a HashMap<ServerID, Addr>,
               leaders: &'a HashSet<ServerID>) -> Self {
        ReplicaNode {
            server: ServerT::bind(addr),
            replica: Replica::new(leaders),
            server_addrs: server_addrs,
            client_type: PhantomData,
        }
    }
}

impl<'a, S, ServerT, ClientT> Node for ReplicaNode<'a, S, ServerT, ClientT> where
    S: StateMachine,
    S::Op: serde::Serialize + serde::de::DeserializeOwned + Clone + Eq + Hash + std::fmt::Debug,
    S::Result: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug,
    ServerT: MsgRecver<Message<S::Op, S::Result>>,
    ClientT: MsgSender<Message<S::Op, S::Result>> {
    type PollItem = ();

    fn non_blocking_processing(&mut self) -> Result<(), i32> {
        let maybe_msg = self.server.try_recv_timeout(100);
        let msg = maybe_msg.unwrap_or(Message::Tick);
        let (to_send_server, to_send_client) = self.replica.handle_msg(&msg);
        to_send_server.into_iter().map(|(server_id, m)| {
            self.server_addrs.get(&server_id).map(|addr| {
                let mut c = ClientT::connect(addr);
                c.send(&m);
            });
        }).collect::<()>();
        to_send_client.into_iter().map(|(addr, m)| {
            let mut c = ClientT::connect(&addr);
            c.send(&m);
        }).collect::<()>();
        Ok(())
    }
}


pub struct ClientNode<'a, S: StateMachine, ServerT, ClientT> {
    server: ServerT,
    replicas: &'a HashSet<Addr>,
    state_machine_type: PhantomData<S>,
    client_type: PhantomData<ClientT>,
}

impl<'a, S, ServerT, ClientT> ClientNode<'a, S, ServerT, ClientT> where
    S: StateMachine,
    S::Op: serde::Serialize + serde::de::DeserializeOwned,
    S::Result: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug, 
    ServerT: MsgRecver<Message<S::Op, S::Result>>,
    ClientT: MsgSender<Message<S::Op, S::Result>> {
    pub fn new(addr: &Addr,
               replicas: &'a HashSet<Addr>) -> Self {
        ClientNode {
            server: ServerT::bind(addr),
            replicas: replicas,
            state_machine_type: PhantomData,
            client_type: PhantomData,
        }
    }

    pub fn send_cmd(&mut self, op: &Message<S::Op, S::Result>) -> Result<S::Result, i32> {
        let addr_vec = self.replicas.iter()
            .map(|a| a.clone())
            .collect::<Vec<_>>();
        let maybe_addr = thread_rng().choose(addr_vec.as_slice());
        maybe_addr.map_or(Err(-1), |addr| {
            let mut c = ClientT::connect(addr);
            c.send(op);
            loop {
            self.server.try_recv_timeout(1000).map_or(Err(-2), |msg| {
                match msg {
                    Message::Response { cid: _, result } => {
                        println!("got result: {:?}", result);
                        Ok(result)
                    },
                    _ => Err(-3),
                }
            });
            }
            Err(-1)
        })
    }
}
