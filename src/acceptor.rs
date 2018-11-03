use std::collections::HashMap;
use messages::*;

pub struct Acceptor<CmdT> {
    ballot: Ballot,
    accepted: HashMap<u64, (Ballot, CmdT)>,
    server_id: ServerID,
}

impl<CmdT> Acceptor<CmdT> where
    CmdT: serde::Serialize + serde::de::DeserializeOwned + Clone {
    pub fn new(my_id: ServerID) -> Self {
        Acceptor {
            ballot: Ballot::bot(my_id),
            accepted: HashMap::new(),
            server_id: my_id,
        }
    }

    pub fn remove_before(&mut self, slot: u64) {
        // remove all accepeted values in range [0, slot)
        self.accepted = std::mem::replace(&mut self.accepted, HashMap::new()).into_iter().filter(|(k, _v)| {
            *k < slot
        }).collect();
    }

    pub fn handle_msg<ResultT>(&mut self, msg: &Message<CmdT, ResultT>) -> Vec<(ServerID, Message<CmdT, ResultT>)> {
        let mut ret: Vec<(ServerID, Message<CmdT, ResultT>)> = Vec::new();
        match msg {
            Message::P1a { sender, ballot } => {
                if *ballot > self.ballot {
                    self.ballot = ballot.clone();
                }
                let proposals = self.accepted.iter().map(|(k, (b, c))| {
                    (*k, b.clone(), c.clone())
                }).collect();
                ret.push((*sender, Message::P1b { sender: self.server_id, ballot: self.ballot.clone(), 
                                                  proposals: proposals }))
            },
            Message::P2a { sender, ballot, slot, cmd } => {
                if *ballot == self.ballot {
                    self.accepted.insert(*slot, (self.ballot.clone(), cmd.clone()));
                } 
                ret.push((*sender, Message::P2b { sender: self.server_id, ballot: self.ballot.clone(), 
                                                  slot: *slot }));
            },
            _ => (),
        };
        ret
    }
}
