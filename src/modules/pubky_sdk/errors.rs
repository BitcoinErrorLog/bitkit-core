//! Error types for pubky SDK FFI

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PubkyError {
    #[error("Authentication failed: {message}")]
    Auth { message: String },
    
    #[error("Network error: {message}")]
    Network { message: String },
    
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
    
    #[error("Session error: {message}")]
    Session { message: String },
    
    #[error("Build error: {message}")]
    Build { message: String },
    
    #[error("Storage error: {message}")]
    Storage { message: String },
    
    #[error("Not found: {message}")]
    NotFound { message: String },
}

impl From<pubky::Error> for PubkyError {
    fn from(err: pubky::Error) -> Self {
        // Convert all error types to string representation for FFI
        PubkyError::Network { 
            message: err.to_string() 
        }
    }
}

impl From<pubky::BuildError> for PubkyError {
    fn from(err: pubky::BuildError) -> Self {
        PubkyError::Build { message: err.to_string() }
    }
}
