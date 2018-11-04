use messages::*;
use statemachine::*;
use std::collections::{ HashSet, HashMap };
use std::hash::Hash;

static WINDOW: u64 = 64;

pub struct Replica<'a, S: StateMachine> {
    state: S,
    slot_in: u64,
    slot_out: u64,
    requests: HashSet<(ClientID, S::Op)>,
    proposals: HashMap<u64, (ClientID, S::Op)>,
    log: HashMap<u64, S::Op>,
    leaders: &'a HashSet<ServerID>,
}

impl<'a, S> Replica<'a, S> where
    S: StateMachine,
    S::Op: Clone + Eq + Hash + std::fmt::Debug,
    S::Result: std::fmt::Debug {

    pub fn new(leaders: &'a HashSet<ServerID>) -> Self {
        Replica {
            state: S::init_state(),
            slot_in: 1,
            slot_out: 1,
            requests: HashSet::new(),
            proposals: HashMap::new(),
            log: HashMap::new(),
            leaders: leaders,
        }
    }

    pub fn handle_msg(&mut self, msg: &Message<S::Op, S::Result>) 
                      -> (Vec<(ServerID, Message<S::Op, S::Result>)>, Vec<(ClientID, Message<S::Op, S::Result>)>) {
        let mut to_server: Vec<(ServerID, Message<S::Op, S::Result>)> = Vec::new();
        let mut to_client: Vec<(ClientID, Message<S::Op, S::Result>)> = Vec::new();
        match msg {
            Message::Request { cid, cmd } => {
                self.requests.insert((cid.clone(), cmd.clone()));
                println!("{} requests: {:?}", std::process::id(), self.requests);
            },
            Message::Decision { slot, cmd } => {
                //println!("{} decided: {} {:?}", std::process::id(), slot, cmd);
                self.log.insert(*slot, cmd.clone());
                to_client.append(&mut self.try_perform());
            },
            _ => (),
        };
        to_server.append(&mut self.propose());
        (to_server, to_client)
    }

    fn try_perform(&mut self) -> Vec<(ClientID, Message<S::Op, S::Result>)> {
        let mut ret: Vec<(ClientID, Message<S::Op, S::Result>)> = Vec::new();
        loop {
            let mut slot_out = self.slot_out;
            let log_ref = &self.log;
            let mut state = std::mem::replace(&mut self.state, S::init_state());
            let cont = log_ref.get(&slot_out).map(|op| {
                println!("{}: applying {}", std::process::id(), slot_out);
                let result = state.apply_op(op);
                assert!(slot_out != std::u64::MAX, "slot number overflow");
                self.proposals.get(&slot_out).map(|(client, _)| {
                    println!("{}: sending reply {:?}", std::process::id(), result);
                    let result_msg = Message::Response { cid: client.clone(), result: result };
                    ret.push((client.clone(), result_msg));
                });
                slot_out += 1;
            });
            self.state = state;
            self.slot_out = slot_out;
            if cont.is_none() {
                break;
            }
        }
        self.proposals = std::mem::replace(&mut self.proposals, HashMap::new())
            .into_iter().filter(|(slot, (cid, op))| {
                let has_key = self.log.contains_key(slot);
                if has_key {
                    // put the proposal back to requests set
                    //self.requests.insert((cid.clone(), op.clone()));
                }
                !has_key
            }).collect();
        ret
    }

    fn propose(&mut self) -> Vec<(ServerID, Message<S::Op, S::Result>)> {
        let requests = std::mem::replace(&mut self.requests, Default::default());
        let upper_bound = self.slot_out + WINDOW;
        let mut ret: Vec<(ServerID, Message<S::Op, S::Result>)> = Vec::new();
        self.requests = requests.into_iter().filter(|(cid, op)| {
            let mut should_keep = true;
            while self.slot_in < upper_bound {
                if !self.log.contains_key(&self.slot_in) {
                    self.leaders.iter().map(|l| {
                        ret.push((l.clone(), Message::Propose { slot: self.slot_in, cmd: op.clone() }));
                    }).collect::<()>();
                    self.proposals.insert(self.slot_in, (cid.clone(), op.clone()));
                    should_keep = false;
                }
                self.slot_in = self.slot_in + 1;
                if !should_keep {
                    break;
                }
            }
            should_keep
        }).collect();
        if self.requests.len() > 0 {
            println!("{} requests after propose: {:?} {:?} {} {}", std::process::id(), self.requests, self.log, self.slot_in, self.slot_out);
        }
        ret
    }
}
