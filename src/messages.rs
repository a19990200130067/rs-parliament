use std::cmp::Ordering;

type ClientID = u64;
type ServerID = u64;


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

#[derive(Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Ballot {
    pub server: ServerID,
    pub idx: u64,
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
