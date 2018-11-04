use std::collections::HashSet;
use std::collections::HashMap;
use std::time::{ Duration, SystemTime };
use messages::*;

pub struct Leader<'a, CmdT> {
    acceptors: &'a HashSet<ServerID>,
    replicas: &'a HashSet<ServerID>,
    waitfor: HashSet<ServerID>,
    is_active: bool,
    ballot: Ballot,
    proposals: HashMap<u64, (CmdT, HashSet<ServerID>)>,
    server_id: ServerID,
    now: SystemTime,
}

impl<'a, CmdT> Leader<'a, CmdT> {
    pub fn new(acceptors: &'a HashSet<ServerID>, replicas: &'a HashSet<ServerID>, my_id: ServerID) -> Self {
        Leader {
            acceptors: acceptors,
            replicas: replicas,
            waitfor: acceptors.clone(),
            is_active: false,
            ballot: Ballot::zero(my_id),
            proposals: HashMap::new(),
            server_id: my_id,
            now: SystemTime::now(),
        }
    }
}

impl<'a, CmdT> Leader<'a, CmdT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone + std::fmt::Debug {
    pub fn remove_before(&mut self, slot: u64) {
        // remove all proposals in range [0, slot)
        self.proposals = std::mem::replace(&mut self.proposals, HashMap::new()).into_iter().filter(|(k, _v)| {
            *k < slot
        }).collect();
    }

    pub fn handle_msg<ResultT>(&mut self, msg: &Message<CmdT, ResultT>) -> Vec<(ServerID, Message<CmdT, ResultT>)> where
        ResultT: std::fmt::Debug {
        let mut ret: Vec<(ServerID, Message<CmdT, ResultT>)> = Vec::new();
        match msg {
            Message::Propose { slot, cmd } => {
                self.proposals.insert(*slot, (cmd.clone(), self.acceptors.clone()));
                if self.is_active {
                    let mut msgs = self.acceptors.iter().map(|server| {
                        (*server, Message::P2a { sender: self.server_id, ballot: self.ballot.clone(), 
                                                 slot: *slot, cmd: cmd.clone() })
                    }).collect();
                    ret.append(&mut msgs);
                }
                println!("{}: proposals: {:?}", std::process::id(), self.proposals);
            },
            Message::P1b { sender, ballot, proposals } => {
                if *ballot == self.ballot && !self.is_active {
                    let mut p_max: HashMap<u64, (&Ballot, &CmdT)> = HashMap::new();
                    proposals.into_iter().map(|(slot, b, c)| {
                        if p_max.contains_key(slot) {
                            p_max.insert(*slot, (b, c));
                        } else {
                            p_max.get_mut(slot).map(|r| {
                                if *b > *r.0 {
                                    r.0 = b;
                                    r.1 = c;
                                }
                            });
                        }
                    }).collect::<()>();
                    p_max.into_iter().map(|(slot, v)| {
                        self.proposals.insert(slot, (v.1.clone(), self.acceptors.clone()));
                    }).collect::<()>();
                    self.waitfor.remove(sender);
                    println!("waitfor of {}: {:?}", self.server_id, self.waitfor);
                    if self.waitfor.len() < self.acceptors.len() / 2 {
                        // got majority vote
                        println!("{} got majority vote", self.server_id);
                        self.waitfor.clear();
                        self.proposals.iter().map(|(slot, (cmd, _))| {
                            let mut msgs = self.acceptors.iter().map(|server| {
                                (*server, Message::P2a { sender: self.server_id, ballot: self.ballot.clone(),
                                                         slot: *slot, cmd: cmd.clone() })
                            }).collect();
                            ret.append(&mut msgs);
                        }).collect::<()>();
                        self.is_active = true;
                    }
                } else {
                    ret.append(&mut self.preempted(ballot))
                }
            },
            Message::P2b { sender, ballot, slot } => {
                if *ballot == self.ballot && self.is_active && self.proposals.contains_key(slot) {
                    let mut proposals = std::mem::replace(&mut self.proposals, HashMap::new());
                    let mut remove_entry = false;
                    proposals.get_mut(slot).map(|(cmd, waitfor)| {
                        waitfor.remove(sender);
                        if waitfor.len() < self.acceptors.len() / 2 {
                            // can make decision, thus remove the entry
                            remove_entry = true;
                            let mut msgs = self.replicas.iter().map(|server| {
                                (*server, Message::Decision { slot: *slot, cmd: cmd.clone()})
                            }).collect();
                            ret.append(&mut msgs);
                        }
                    });
                    if remove_entry {
                        proposals.remove(slot);
                    }
                    self.proposals = proposals;
                } else {
                    ret.append(&mut self.preempted(ballot))
                }
            },
            _ => (),
        };
        if !self.proposals.is_empty() {
            match self.now.elapsed() {
                Ok(elapsed) => {
                    if elapsed.as_secs() > 0 {
                        let mut msgs = self.acceptors.iter().map(|server| {
                            (*server, Message::P1a { sender: self.server_id, ballot: self.ballot.clone() })
                        }).collect();
                        println!("{} p1a : {:?}", std::process::id(), msgs);
                        ret.append(&mut msgs);
                        self.now = SystemTime::now();
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        } else {
            self.now = SystemTime::now();
        }
        ret
    }

    fn preempted<ResultT>(&mut self, b: &Ballot) -> Vec<(ServerID, Message<CmdT, ResultT>)> {
        if *b > self.ballot {
            self.is_active = false;
            self.ballot = self.ballot.next().expect("ballot reaches maximum");
            self.waitfor = self.acceptors.clone();
            self.acceptors.iter().map(|server| {
                (*server, Message::P1a { sender: self.server_id, ballot: self.ballot.clone() })
            }).collect()
        } else {
            Vec::new()
        }
    }
}
