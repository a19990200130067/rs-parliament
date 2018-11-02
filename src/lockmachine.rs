use std::collections::HashMap;
use statemachine::*;

#[derive(Serialize, Deserialize)]
pub enum LockOp {
    TryLock(u64, u64),
    TryUnlock(u64, u64),
}

#[derive(Serialize, Deserialize)]
pub enum LockResult {
    Success,
    Fail,
}

pub struct LockMachine {
    locks: HashMap<u64, u64>,
}

impl StateMachine for LockMachine {
    type Op = LockOp;
    type Result = LockResult;

    fn apply_op(&mut self, op: &Self::Op) -> Self::Result {
        match op {
            LockOp::TryLock(lockid, cid) => {
                let mut maybe_c: Option<u64>;
                {
                    maybe_c = locks.get(lockid).map(|c| *c);
                }

                maybe_c.map_or_else(|| {
                    locks.insert(*lockid, *cid);
                    LockResult::Success
                }, |c| {
                    if c == *cid {
                        LockResult::Success
                    } else {
                        LockResult::Fail
                    }
                })
            },
            LockOp::TryUnlock(lockid, cid) => {
                let mut c: u64;
                {
                    let maybe_c = locks.get(lockid);
                    if maybe_c.is_none() {
                        return LockResult::Fail
                    }
                    c = maybe_c.unwrap().clone();
                }
                if c == *cid {
                    locks.remove(lockid);
                    LockResult::Success
                } else {
                    LockResult::Fail
                }
            },
        }
    }
}
