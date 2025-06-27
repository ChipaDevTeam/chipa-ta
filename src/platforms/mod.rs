#[cfg(feature = "pocket_options")]
pub mod pocket_option;

#[cfg(feature = "pocket_options")]
pub use pocket_option::PocketOptionPlatform;

#[derive(thiserror::Error, Debug)]
pub enum PlatformError {
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Platform not initialized")]
    NotInitialized,

    #[cfg(feature = "pocket_options")]
    #[error("PocketOption error: {0}")]
    PocketOptionError(#[from] binary_options_tools::pocketoption::error::PocketOptionError),
}

