use std::cmp::Ordering;
use messaging::Addr;

pub type ClientID = Addr;
pub type ServerID = u64;

#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Ballot {
    pub server: ServerID,
    pub idx: u64,
    is_bot: bool, 
}

impl Ballot {
    pub fn next(&self) -> Option<Self> {
        if self.idx != std::u64::MAX {
            Some(Ballot { server: self.server, idx: self.idx + 1, is_bot: false })
        } else {
            None
        }
    }
    
    pub fn zero(self_id: ServerID) -> Self {
        Ballot { server: self_id, idx: 0, is_bot: false }
    }

    pub fn bot(self_id: ServerID) -> Self {
        Ballot { server: self_id, idx: 0, is_bot: true }
    }
}

impl PartialOrd for Ballot {
    fn partial_cmp(&self, other: &Ballot) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ballot {
    fn cmp(&self, other: &Ballot) -> Ordering {
        if self.is_bot {
            if other.is_bot {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        } else {
            if other.is_bot {
                Ordering::Greater
            } else {
                let idx_cmp = self.idx.cmp(&other.idx);
                if idx_cmp == Ordering::Equal {
                    self.server.cmp(&other.server)
                } else {
                    idx_cmp
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message<CmdT, ResultT> {
    Request { cid: ClientID, cmd: CmdT },
    Response { cid: ClientID, result: ResultT },

    Propose { slot: u64, cmd: CmdT },
    Adopted { slot: u64, ballot: Ballot, cmd: CmdT },
    Decision { slot: u64, cmd: CmdT },
    
    P1a { sender: ServerID, ballot: Ballot },
    P1b { sender: ServerID, ballot: Ballot, proposals: Vec<(u64, Ballot, CmdT)> },
    P2a { sender: ServerID, ballot: Ballot, slot: u64, cmd: CmdT },
    P2b { sender: ServerID, ballot: Ballot, slot: u64 },
}

/*
#[derive(Serialize, Deserialize)]
pub struct Propose<CmdT> {
    pub slot_in: u64,
    pub cmd: CmdT,
}

#[derive(Serialize, Deserialize)]
pub struct Response<ResultT> {
    pub cid: ClientID,
    pub result: ResultT,
}

#[derive(Serialize, Deserialize)]
pub struct Request<CmdT> {
    pub cmd: CmdT,
}

#[derive(Serialize, Deserialize)]
pub struct Decision<CmdT> {
    pub slot: u64,
    pub cmd: CmdT,
}

#[derive(Serialize, Deserialize)]
pub struct P1a {
    pub sender: ServerID,
    pub ballot: Ballot,
}

#[derive(Serialize, Deserialize)]
pub struct P1b<CmdT> {
    pub sender: ServerID,
    pub ballot: Ballot,
    pub slot_id: u64,
    pub cmd: CmdT,
}

#[derive(Serialize, Deserialize)]
pub struct P2a<CmdT> {
    pub sender: ServerID,
    pub ballot: Ballot,
    pub slot_id: u64,
    pub cmd: CmdT,
}

#[derive(Serialize, Deserialize)]
pub struct P2b {
    pub sender: ServerID,
    pub ballot: Ballot,
}
*/
