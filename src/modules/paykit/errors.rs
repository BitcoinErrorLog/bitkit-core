use thiserror::Error;

#[derive(uniffi::Error, Debug, Error)]
#[non_exhaustive]
pub enum PaykitError {
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Invalid public key format: {0}")]
    InvalidPublicKey(String),
    #[error("Operation not supported: {0}")]
    Unsupported(String),
    #[error("Generic error: {0}")]
    Generic(String),
}

impl From<paykit_lib::PaykitError> for PaykitError {
    fn from(err: paykit_lib::PaykitError) -> Self {
        // Convert all paykit_lib error variants to our simplified error type
        PaykitError::Generic(err.to_string())
    }
}

impl From<pubky::Error> for PaykitError {
    fn from(err: pubky::Error) -> Self {
        PaykitError::Transport(err.to_string())
    }
}
