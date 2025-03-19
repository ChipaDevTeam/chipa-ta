pub mod error;
pub mod helper;
/// This is a Technical analysis crate based on [`ta-rs`](https://github.com/greyblake/ta-rs) and [`rust_ti`](https://github.com/0100101001010000/RustTI)
pub mod indicators;
pub mod traits;
pub mod types;

pub mod helper_types;

pub use indicators::Indicator;

#[cfg(feature = "js")]
pub use indicators::js::{CandleJs, IndicatorJs};

#[cfg(feature="py")]
mod py {
    use pyo3::{pymodule, types::{PyModule, PyModuleMethods}, Bound, PyResult};
    use pyo3_stub_gen::define_stub_info_gatherer;

    use crate::indicators::py::{Candle, Indicator};
    
    #[pymodule]
    #[pyo3(name = "chipa_ta")]
    fn chipa_ta(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<Indicator>()?;
        m.add_class::<Candle>()?;

        Ok(())
    }

    define_stub_info_gatherer!(stub_info);
}

#[cfg(feature="py")]
pub use py::*;