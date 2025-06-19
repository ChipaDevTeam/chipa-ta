use serde::{Deserialize, Serialize};

use crate::traits::Reset;

/// Preprocessing steps applied to market data before evaluation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PreprocessingStep {
    /// Wavelet denoise step.
    WaveletDenoise,
    /// Normalize data step.
    Normalize,
    // TODO: Add more preprocessing steps.
}

impl PreprocessingStep {
    /// Applies the preprocessing step to market data.
    ///
    /// NOTE: This is a placeholder implementation. Replace with actual logic.
    pub fn apply(&self, _data: &mut crate::strategy::MarketData) {
        match self {
            PreprocessingStep::WaveletDenoise => {
                // TODO: Implement wavelet denoising on data
                // For now, do nothing.
            }
            PreprocessingStep::Normalize => {
                // TODO: Implement normalization on data
                // For now, do nothing.
            }
        }
    }
}

impl Reset for PreprocessingStep {
    fn reset(&mut self) {
        // Reset logic for preprocessing steps if needed.
        // Currently, no state to reset, so this is a no-op.
    }
}
