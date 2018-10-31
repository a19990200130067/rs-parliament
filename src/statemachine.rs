extern crate serde;

use std::default::Default;

pub trait StateMachine {
    type Op;
    type Result;
    type State: Default;

    fn init_state() -> Self::State {
        Default::default()
    }
    fn apply_op(state: &mut Self::State, op: &Self::Op) -> Self::Result;
}
