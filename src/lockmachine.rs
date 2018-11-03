use std::collections::HashMap;
use statemachine::*;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum LockOp {
    TryLock(u64, u64),
    TryUnlock(u64, u64),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LockResult {
    Success,
    Fail,
}

#[derive(Default)]
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
                    maybe_c = self.locks.get(lockid).map(|c| *c);
                }

                maybe_c.map_or_else(|| {
                    self.locks.insert(*lockid, *cid);
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
                    let maybe_c = self.locks.get(lockid);
                    if maybe_c.is_none() {
                        return LockResult::Fail
                    }
                    c = maybe_c.unwrap().clone();
                }
                if c == *cid {
                    self.locks.remove(lockid);
                    LockResult::Success
                } else {
                    LockResult::Fail
                }
            },
        }
    }
}
