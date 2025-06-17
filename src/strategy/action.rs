use serde::{Deserialize, Serialize};

/// Actions that a strategy can take after evaluation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Submit a buy order or signal.
    Buy,
    /// Submit a sell order or signal.
    Sell,
    /// Do nothing or hold position.
    Hold,
}
