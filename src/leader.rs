use std::collections::HashSet;
use std::collections::HashMap;
use messages::*;

pub struct Leader<'a, CmdT> {
    acceptors: &'a HashSet<ServerID>,
    replicas: &'a HashSet<ServerID>,
    waitfor: HashSet<ServerID>,
    is_active: bool,
    ballot: Ballot,
    proposals: HashMap<u64, (CmdT, HashSet<ServerID>)>,
    server_id: ServerID,
}

impl <'a, CmdT> Leader<'a, CmdT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone {
    pub fn remove_before(&mut self, slot: u64) {
        // remove all proposals in range [0, slot)
        self.proposals = std::mem::replace(&mut self.proposals, HashMap::new()).into_iter().filter(|(k, _v)| {
            *k < slot
        }).collect();
    }

    pub fn handle_msg<ResultT>(&mut self, msg: &Message<CmdT, ResultT>) -> Vec<(ServerID, Message<CmdT, ResultT>)> {
        match msg {
            Message::Propose { slot, cmd } => {
                self.proposals.insert(*slot, (cmd.clone(), self.acceptors.clone()));
                self.acceptors.iter().map(|server| {
                    (*server, Message::P2a { sender: self.server_id, ballot: self.ballot.clone(), 
                                             slot: *slot, cmd: cmd.clone() })
                }).collect()
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
                    if self.waitfor.len() < self.acceptors.len() / 2 {
                        // got majority vote
                        self.waitfor.clear();
                        self.is_active = true;
                    }
                } else {
                    self.preempted(ballot);
                }
                Vec::new()
            },
            Message::P2b { sender, ballot, slot } => {
                if *ballot == self.ballot && self.is_active && self.proposals.contains_key(slot) {
                    let mut proposals = std::mem::replace(&mut self.proposals, HashMap::new());
                    let ret = proposals.get_mut(slot).map_or(Vec::new(), |(cmd, waitfor)| {
                        waitfor.remove(sender);
                        if waitfor.len() < self.acceptors.len() / 2 {
                            self.replicas.iter().map(|server| {
                                (*server, Message::Decision { slot: *slot, cmd: cmd.clone()})
                            }).collect()
                        } else {
                            Vec::new()
                        }
                    });
                    self.proposals = proposals;
                    ret
                } else {
                    self.preempted(ballot);
                    Vec::new()
                }
            },
            _ => Vec::new(),
        }
    }

    fn preempted(&mut self, b: &Ballot) {
        if *b > self.ballot {
            self.is_active = false;
            self.ballot = self.ballot.next().expect("ballot reaches maximum");
            self.waitfor = self.acceptors.clone();
        }
    }
}
