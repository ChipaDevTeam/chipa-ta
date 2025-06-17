use crate::{
    error::TaResult,
    strategy::MarketData,
    traits::{Next, Period},
    types::OutputType,
    Indicator,
};
use serde::{Deserialize, Serialize};

/// Logical conditions for strategy execution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Condition {
    /// Checks if an indicator value is greater than a threshold.
    GreaterThan {
        indicator: Indicator,
        value: OutputType,
    },
    /// Checks if an indicator value is less than a threshold.
    LessThan {
        indicator: Indicator,
        value: OutputType,
    },
    /// Logical AND of multiple conditions.
    And(Vec<Condition>),
    /// Logical OR of multiple conditions.
    Or(Vec<Condition>),
    /// Logical NOT of a condition.
    Not(Box<Condition>),
}

impl Condition {
    /// Evaluate the condition against market data.
    pub fn evaluate(&mut self, data: &MarketData) -> TaResult<bool> {
        match self {
            Condition::GreaterThan { indicator, value } => {
                indicator.next(data)?.cmp_output(value, |x, y| x > y)
            }
            Condition::LessThan { indicator, value } => {
                indicator.next(data)?.cmp_output(value, |x, y| x < y)
            }
            Condition::And(conds) => {
                for c in conds.iter_mut() {
                    if !c.evaluate(data)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Or(conds) => {
                for c in conds.iter_mut() {
                    if c.evaluate(data)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Condition::Not(c) => Ok(!c.evaluate(data)?),
        }
    }

    /// Returns the maximum indicator period contained in this condition or `None` if no indicators.
    pub fn max_period(&self) -> Option<usize> {
        match self {
            Condition::GreaterThan { indicator, .. } | Condition::LessThan { indicator, .. } => {
                Some(indicator.period())
            }
            Condition::And(conds) | Condition::Or(conds) => {
                conds.iter().filter_map(|c| c.max_period()).max()
            }
            Condition::Not(cond) => cond.max_period(),
        }
    }
}
