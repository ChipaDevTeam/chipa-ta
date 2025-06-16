//! Procedural macro to generate a Python-compatible struct from an enum like `Indicator`.
//!
//! This macro will:
//! - Take an enum (e.g., `Indicator`) and generate a struct (e.g., `IndicatorPy`) with the same variants.
//! - Decorate the struct and its methods with `#[pyclass]`, `#[gen_stub_pyclass]`, etc. for pyo3 and pyo3-stubgen compatibility.
//! - Generate Python constructors for each variant, copying the arguments from the original enum's constructors.
//! - Implement `next`, `next_batched`, `next_candle`, and `next_candles` as Python-callable methods.
//! - Be extensible and well-documented for future updates.

mod indicator_py_macro;

// Re-export the macro for use in other crates
pub use indicator_py_macro::indicator_py;
