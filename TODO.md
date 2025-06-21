# Project TODOs

This document tracks outstanding tasks and ideas for the `preprocessing`, `indicators`, `strategy`, and general project improvements. It includes explicit `TODO`/`FIXME` comments found in the codebase and additional suggestions.

---

## General

- [ ] **Indicator Readiness Wrapper**  
      Implement a function, trait, or wrapper for indicators (or the `Next` trait) that:
  - Returns `Result<Option<OutputType>>` from `next` while the indicator is not "ready" (i.e., not enough data has been passed, less than its period).
  - Returns `Result<Some(OutputType)>` when the indicator is ready.
  - This could be a trait extension, a decorator/wrapper struct, or a change to the indicator interface.
- [ ] Review and address all `TODO` and `FIXME` comments across the codebase.

---

## Preprocessing

- [ ] **Wavelet Denoising Preprocessing**

  - Implement a wavelet-based denoising step in `PreprocessingStep::WaveletDenoise`.
  - Integrate a wavelet transform library or implement a simple wavelet filter.
  - Replace the placeholder in `PreprocessingStep::apply` for `WaveletDenoise` with actual logic.

- [ ] **Add More Preprocessing Steps**
  - Expand the enum and logic for additional preprocessing methods as needed.

---

## Indicators

- [ ] **Indicator Output Readiness**

  - Ensure all indicators consistently handle "not enough data" situations (see General section).
  - Consider adding a method like `is_ready()` to all indicators, or use the wrapper approach.

- [ ] **Alligator Indicator**

  - Ensure the new Alligator indicator is fully integrated and tested.
  - Add serialization/deserialization support if needed.

- [ ] **SMMA Minimum Period**

  - Enforce a minimum period of 2 for the Smoothed Moving Average (SMMA) indicator.

- [ ] **Indicator Comparison**
  - Ensure the new `Condition::Indicator` logic (indicator vs indicator) is fully supported and tested.

---

## Strategy

- [ ] **Indicator vs Indicator Conditions**

  - Confirm that the new `Condition::Indicator { left, right, operator }` is used throughout and tested.
  - Update any strategy logic or UI that previously assumed only indicator vs value.

- [ ] **Stop Loss, Take Profit, Martingale**
  - Consider adding support for stop loss, take profit, and martingale logic in strategies.
  - This may require trade state tracking and additional strategy node types.

---

## Other

- [ ] **Documentation**

  - Update documentation and code comments to reflect new features and changes.
  - Add usage examples for new indicators and strategy features.

- [ ] **Testing**
  - Add or expand unit tests for all new features, especially for indicator readiness and new preprocessing steps.

---

## Codebase TODOs/FIXMEs (from source)

- [ ] `src/preprocessing/mod.rs`:

  - `// TODO: Add more preprocessing steps.`
  - `// TODO: Implement wavelet denoising on data`
  - `// TODO: Implement normalization on data`

- [ ] `src/types.rs`:
  - `// TODO: Implement PartialEq and PartialOrd for OutputType using std::f64::EPSILON`

---

Add new items here as you discover more areas for improvement!
