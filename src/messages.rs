pub type ClientID = u64;
pub type ServerID = u64;

#[derive(Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Clone)]
pub struct Ballot {
    pub server: ServerID,
    pub idx: u64,
}

impl Ballot {
    pub fn next(&self) -> Option<Self> {
        if self.idx != std::u64::MAX {
            Some(Ballot { server: self.server, idx: self.idx + 1 })
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Message<CmdT, ResultT> {
    Request { cmd: CmdT },
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
