use crate::traits::Candle;

#[derive(Debug, PartialEq, Clone)]
pub struct Bar {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    price: f64,
    volume: f64,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            open: 0.0,
            close: 0.0,
            low: 0.0,
            high: 0.0,
            price: 0.0,
            volume: 0.0,
        }
    }
}

impl Bar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_open<T: Into<f64>>(mut self, val: T) -> Self {
        self.open = val.into();
        self
    }

    pub fn set_high<T: Into<f64>>(mut self, val: T) -> Self {
        self.high = val.into();
        self
    }

    pub fn set_low<T: Into<f64>>(mut self, val: T) -> Self {
        self.low = val.into();
        self
    }

    pub fn set_close<T: Into<f64>>(mut self, val: T) -> Self {
        self.close = val.into();
        self
    }

    pub fn set_price<T: Into<f64>>(mut self, val: T) -> Self {
        self.price = val.into();
        self
    }

    pub fn set_volume(mut self, val: f64) -> Self {
        self.volume = val;
        self
    }
}

impl Candle for Bar {
    fn close(&self) -> f64 {
        self.close
    }

    fn open(&self) -> f64 {
        self.open
    }

    fn high(&self) -> f64 {
        self.high
    }

    fn low(&self) -> f64 {
        self.low
    }

    fn price(&self) -> f64 {
        self.price
    }

    fn volume(&self) -> f64 {
        self.volume
    }
}

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
            return Err(crate::error::TaError::InvalidParameter("0".to_string()));
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
    pub fn new(capacity: usize) -> TaResult<Self> {
        if capacity == 0 {
            return Err(crate::error::TaError::InvalidParameter("0".to_string()));
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
