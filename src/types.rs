use core::fmt::Debug;
use std::ops::{Deref, DerefMut};

use chipa_ta_utils::TaUtilsError;
use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    traits::{Period, Reset},
};

pub use chipa_ta_utils::{OutputError, OutputShape, OutputType, Statics};

// Can you help me emprove the Queue struct? the goal is to make it like a Vec but with a fixed capacity that removes the oldest element when a new one is added beyond its capacity.
// it also implements the Period and Reset traits, allowing it to be used in a similar way to Cycle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Queue<T> {
    queue: Vec<T>,
    period: usize,
}

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

impl<T: Default + Clone> Queue<T> {
    pub fn new(period: usize) -> TaResult<Self> {
        if period == 0 {
            return Err(TaUtilsError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ).into());
        }
        Ok(Self {
            queue: Vec::with_capacity(period),
            period,
        })
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        self.queue.push(value);
        if self.queue.len() > self.period {
            let removed = self.queue.remove(0);
            Some(removed)
        } else {
            None
        }
    }
}

impl<T> Deref for Queue<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl<T> DerefMut for Queue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.queue
    }
}

impl<T> Period for Queue<T> {
    fn period(&self) -> usize {
        self.period
    }
}
impl<T> Reset for Queue<T> {
    fn reset(&mut self) {
        self.queue = Vec::with_capacity(self.period);
    }
}
