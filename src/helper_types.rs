pub use chipa_ta_utils::Bar;
use chipa_ta_utils::{TaUtilsError, TaUtilsResult};

use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    traits::{Period, Reset},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Cycle {
    period: usize,
    index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Queue<T> {
    queue: VecDeque<T>,
    period: usize,
}

impl Period for Cycle {
    fn period(&self) -> usize {
        self.period
    }
}

impl<T> Period for Queue<T> {
    fn period(&self) -> usize {
        self.period
    }
}

impl Reset for Cycle {
    fn reset(&mut self) {
        self.index = 0;
    }
}

impl<T> Reset for Queue<T> {
    fn reset(&mut self) {
        self.queue = VecDeque::with_capacity(self.period);
    }
}

impl Cycle {
    pub fn new(period: usize) -> TaResult<Self> {
        if period == 0 {
            return Err(TaUtilsError::InvalidParameter("0".to_string()).into());
        }
        Ok(Self { period, index: 0 })
    }

    pub fn next_idx(&mut self) -> usize {
        self.next_silence();
        self.index
    }

    pub fn next_silence(&mut self) {
        if self.index + 1 < self.period {
            self.index += 1;
        } else {
            self.index = 0;
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

impl<T> Queue<T> {
    pub fn new(capacity: usize) -> TaUtilsResult<Self> {
        if capacity == 0 {
            return Err(TaUtilsError::InvalidParameter("0".to_string()).into());
        }
        Ok(Self {
            period: capacity,
            queue: VecDeque::with_capacity(capacity),
        })
    }

    #[inline]
    pub fn next_with(&mut self, value: T) -> Option<T> {
        self.queue.push_back(value);
        if self.queue.len() > self.period {
            return self.queue.pop_front();
        }
        None
    }
}

impl<T> Deref for Queue<T> {
    type Target = VecDeque<T>;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl<T> DerefMut for Queue<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_overflow() {
        let mut queue = Queue::new(10).unwrap();
        for i in 0..12 {
            queue.push_back(i);
        }
        dbg!(&queue);
        queue.pop_front();
        dbg!(&queue);
        assert!(queue.len() == 11)
    }
}
