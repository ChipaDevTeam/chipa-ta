use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    error::TaResult,
    helper::max3,
    traits::{Candle, Indicator, Next, Period, Reset},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrueRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    prev_close: Option<f64>,
}

impl Indicator for TrueRange {}

impl TrueRange {
    pub fn new() -> Self {
        Self::default()
    }
}

impl fmt::Display for TrueRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TRUE_RANGE()")
    }
}

impl Next<f64> for TrueRange {
    type Output = f64;

    fn next(&mut self, input: f64) -> TaResult<Self::Output> {
        let distance = match self.prev_close {
            Some(prev) => (input - prev).abs(),
            None => 0.0,
        };
        self.prev_close = Some(input);
        Ok(distance)
    }
}

impl<T: Candle> Next<&T> for TrueRange {
    type Output = f64;

    fn next(&mut self, bar: &T) -> TaResult<Self::Output> {
        let max_dist = match self.prev_close {
            Some(prev_close) => {
                let dist1 = bar.high() - bar.low();
                let dist2 = (bar.high() - prev_close).abs();
                let dist3 = (bar.low() - prev_close).abs();
                max3(dist1, dist2, dist3)
            }
            None => bar.high() - bar.low(),
        };
        self.prev_close = Some(bar.close());
        Ok(max_dist)
    }
}

impl Reset for TrueRange {
    fn reset(&mut self) {
        self.prev_close = None;
    }
}

impl Period for TrueRange {
    fn period(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use crate::{helper::round, helper_types::Bar};

    use super::*;

    #[test]
    fn test_next_f64() {
        let mut tr = TrueRange::new();
        assert_eq!(round(tr.next(2.5).unwrap()), 0.0);
        assert_eq!(round(tr.next(3.6).unwrap()), 1.1);
        assert_eq!(round(tr.next(3.3).unwrap()), 0.3);
    }

    #[test]
    fn test_next_bar() {
        let mut tr = TrueRange::new();

        let bar1 = Bar::new().set_high(10).set_low(7.5).set_close(9);
        let bar2 = Bar::new().set_high(11).set_low(9).set_close(9.5);
        let bar3 = Bar::new().set_high(9).set_low(5).set_close(8);

        assert_eq!(tr.next(&bar1).unwrap(), 2.5);
        assert_eq!(tr.next(&bar2).unwrap(), 2.0);
        assert_eq!(tr.next(&bar3).unwrap(), 4.5);
    }

    #[test]
    fn test_reset() {
        let mut tr = TrueRange::new();

        let bar1 = Bar::new().set_high(10).set_low(7.5).set_close(9);
        let bar2 = Bar::new().set_high(11).set_low(9).set_close(9.5);

        tr.next(&bar1).unwrap();
        tr.next(&bar2).unwrap();

        tr.reset();
        let bar3 = Bar::new().set_high(60).set_low(15).set_close(51);
        assert_eq!(tr.next(&bar3).unwrap(), 45.0);
    }

    #[test]
    fn test_default() {
        TrueRange::default();
    }

    #[test]
    fn test_display() {
        let indicator = TrueRange::new();
        assert_eq!(format!("{}", indicator), "TRUE_RANGE()");
    }

    #[test]
    fn test_serialize1() {
        let tr = TrueRange::new();
        let tr_string: String = serde_json::to_string(&tr).unwrap();
        assert_eq!(tr_string, r#"{}"#)
    }

    #[test]
    fn test_serialize2() {
        let mut tr = TrueRange::new();
        tr.next(12.1).unwrap();
        let tr_string: String = serde_json::to_string(&tr).unwrap();
        assert_eq!(tr_string, r#"{"prev_close":12.1}"#)
    }

    #[test]
    fn test_deserialize() {
        let tr_string = r#"{}"#;
        let tr_check = TrueRange::new();
        let tr_deserialized: TrueRange = serde_json::from_str(tr_string).unwrap();
        assert_eq!(tr_deserialized, tr_check)
    }
}
