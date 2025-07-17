use core::fmt;
use serde::{Deserialize, Serialize};

use crate::{
    error::{TaError, TaResult},
    indicators::smma::SmoothedMovingAverage,
    traits::{Candle, IndicatorTrait, Next, Period, Reset},
    types::OutputShape,
    types::Queue,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Alligator {
    jaw: SmoothedMovingAverage,
    teeth: SmoothedMovingAverage,
    lips: SmoothedMovingAverage,
    jaw_buffer: Queue<f64>,
    teeth_buffer: Queue<f64>,
    lips_buffer: Queue<f64>,
}

/// Custom implementation of the Serialize and Deserialize traits for Alligator
impl Serialize for Alligator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct AlligatorVisitor {
            jaw_period: usize,
            jaw_shift: usize,
            teeth_period: usize,
            teeth_shift: usize,
            lips_period: usize,
            lips_shift: usize,
        }
        AlligatorVisitor {
            jaw_period: self.jaw.period(),
            jaw_shift: self.jaw_buffer.period(),
            teeth_period: self.teeth.period(),
            teeth_shift: self.teeth_buffer.period(),
            lips_period: self.lips.period(),
            lips_shift: self.lips_buffer.period(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Alligator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AlligatorVisitor {
            jaw_period: usize,
            jaw_shift: usize,
            teeth_period: usize,
            teeth_shift: usize,
            lips_period: usize,
            lips_shift: usize,
        }
        let visitor = AlligatorVisitor::deserialize(deserializer)?;
        Self::new(
            visitor.jaw_period,
            visitor.jaw_shift,
            visitor.teeth_period,
            visitor.teeth_shift,
            visitor.lips_period,
            visitor.lips_shift,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl Default for Alligator {
    fn default() -> Self {
        Self::standard().unwrap()
    }
}

impl Alligator {
    pub fn new(
        jaw_period: usize,
        jaw_shift: usize,
        teeth_period: usize,
        teeth_shift: usize,
        lips_period: usize,
        lips_shift: usize,
    ) -> TaResult<Self> {
        if jaw_period < 2 || teeth_period < 2 || lips_period < 2 {
            return Err(TaError::InvalidParameter(
                "Periods must be at least 2".to_string(),
            ));
        }
        if jaw_shift < 1 || teeth_shift < 1 || lips_shift < 1 {
            return Err(TaError::InvalidParameter(
                "Shifts must be at least 1".to_string(),
            ));
        }
        Ok(Self {
            jaw: SmoothedMovingAverage::new(jaw_period)?,
            teeth: SmoothedMovingAverage::new(teeth_period)?,
            lips: SmoothedMovingAverage::new(lips_period)?,
            jaw_buffer: Queue::new(jaw_shift)?,
            teeth_buffer: Queue::new(teeth_shift)?,
            lips_buffer: Queue::new(lips_shift)?,
        })
    }

    /// Standard Alligator: jaw(13,8), teeth(8,5), lips(5,3)
    pub fn standard() -> TaResult<Self> {
        Self::new(13, 8, 8, 5, 5, 3)
    }
}

impl IndicatorTrait for Alligator {
    fn output_shape(&self) -> OutputShape {
        OutputShape::Shape(3)
    }
}

impl Period for Alligator {
    fn period(&self) -> usize {
        // The max of (period + shift) for each line
        (self.jaw.period() + self.jaw_buffer.period())
            .max(self.teeth.period() + self.teeth_buffer.period())
            .max(self.lips.period() + self.lips_buffer.period())
    }
}

impl Next<f64> for Alligator {
    type Output = (f64, f64, f64);

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let jaw_val = self.jaw.next(input)?;
        let teeth_val = self.teeth.next(input)?;
        let lips_val = self.lips.next(input)?;

        let jaw_shifted = match self.jaw_buffer.push(jaw_val) {
            Some(val) => val,
            None => *self.jaw_buffer.first().unwrap_or(&0.0),
        };
        let teeth_shifted = match self.teeth_buffer.push(teeth_val) {
            Some(val) => val,
            None => *self.teeth_buffer.first().unwrap_or(&0.0),
        };
        let lips_shifted = match self.lips_buffer.push(lips_val) {
            Some(val) => val,
            None => *self.lips_buffer.first().unwrap_or(&0.0),
        };

        Ok((jaw_shifted, teeth_shifted, lips_shifted))
    }
}

impl<T: Candle> Next<&T> for Alligator {
    type Output = (f64, f64, f64);

    fn next(&mut self, input: &T) -> TaResult<Self::Output> {
        // Use median price as input: (high + low) / 2
        let price = (input.high() + input.low()) / 2.0;
        self.next(price)
    }
}

impl Reset for Alligator {
    fn reset(&mut self) {
        self.jaw.reset();
        self.teeth.reset();
        self.lips.reset();
        self.jaw_buffer = Queue::new(self.jaw_buffer.period()).unwrap();
        self.teeth_buffer = Queue::new(self.teeth_buffer.period()).unwrap();
        self.lips_buffer = Queue::new(self.lips_buffer.period()).unwrap();
    }
}


impl fmt::Display for Alligator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Alligator(jaw: SMMA({},+{}), teeth: SMMA({},+{}), lips: SMMA({},+{}))",
            self.jaw.period(),
            self.jaw_buffer.period(),
            self.teeth.period(),
            self.teeth_buffer.period(),
            self.lips.period(),
            self.lips_buffer.period()
        )
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_alligator_basic() {
    //     let mut alligator = Alligator::standard().unwrap();
    //     // Feed in a ramp of values
    //     for i in 1..30 {
    //         let (jaw, teeth, lips) = alligator.next(i as f64).unwrap();
    //         // Before enough data, outputs will be None
    //         // After enough data, outputs will be Some(f64)
    //         // if i < 13 + 8 {
    //         //     assert!(jaw.is_none());
    //         // }
    //         // if i < 8 + 5 {
    //         //     assert!(teeth.is_none());
    //         // }
    //         // if i < 5 + 3 {
    //         //     assert!(lips.is_none());
    //         // }
    //     }
    // }
}