extern crate serde;

use std::default::Default;

pub trait StateMachine : Default{
    type Op;
    type Result;

    fn init_state() -> Self {
        Default::default()
    }
    fn apply_op(&mut self, op: &Self::Op) -> Self::Result;
}
