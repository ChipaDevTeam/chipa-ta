use serde::{Deserialize, Serialize};

/// Actions that a strategy can take after evaluation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Submit a buy order or signal with strong conviction.
    StrongBuy,
    /// Submit a buy order or signal with normal conviction.
    Buy,
    /// Do nothing or hold position.
    Hold,
    /// Submit a sell order or signal with normal conviction.
    Sell,
    /// Submit a sell order or signal with strong conviction.
    StrongSell,
}
