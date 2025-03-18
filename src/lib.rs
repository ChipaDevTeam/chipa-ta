pub mod error;
pub mod helper;
/// This is a Technical analysis crate based on [`ta-rs`](https://github.com/greyblake/ta-rs) and [`rust_ti`](https://github.com/0100101001010000/RustTI)
pub mod indicators;
pub mod traits;
pub mod types;

pub mod helper_types;

#[cfg(feature = "js")]
pub use indicators::js::{CandleJs, IndicatorJs};
