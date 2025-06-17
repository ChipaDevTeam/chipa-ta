use serde::{Deserialize, Serialize};

/// Preprocessing steps applied to market data before evaluation.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub fn apply(&self, data: &mut crate::strategy::MarketData) {
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
