use core::fmt::Debug;

use serde::{Deserialize, Serialize};

pub use chipa_ta_utils::{OutputError, OutputShape, OutputType, Statics, Queue};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status<T, U, V> {
    Initial(T),
    Progress(U),
    Completed(V),
}


impl<T: Default, U, V> Default for Status<T, U, V> {
    fn default() -> Self {
        Self::Initial(T::default())
    }
}

