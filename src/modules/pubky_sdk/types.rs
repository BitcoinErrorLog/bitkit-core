//! FFI-compatible types for pubky SDK

/// Session information returned from sign-in/sign-up
#[derive(Debug, Clone, uniffi::Record)]
pub struct PubkySessionInfo {
    /// The public key of the authenticated user
    pub pubkey: String,
    /// List of granted capabilities
    pub capabilities: Vec<String>,
    /// When the session was created (Unix timestamp in seconds)
    pub created_at: u64,
}

/// A file or directory in storage
#[derive(Debug, Clone, uniffi::Record)]
pub struct PubkyListItem {
    /// Name of the file or directory
    pub name: String,
    /// Full path
    pub path: String,
    /// Whether this is a directory
    pub is_directory: bool,
}

/// Profile information from pubky.app
#[derive(Debug, Clone, uniffi::Record)]
pub struct PubkyProfile {
    /// Display name
    pub name: Option<String>,
    /// Bio/description
    pub bio: Option<String>,
    /// Profile image URL
    pub image: Option<String>,
    /// Links
    pub links: Vec<String>,
    /// Status message
    pub status: Option<String>,
}

/// Options for signing up
#[derive(Debug, Clone, uniffi::Record)]
pub struct PubkySignupOptions {
    /// Signup token if required by homeserver
    pub signup_token: Option<String>,
}

/// Result of generating a keypair
#[derive(Debug, Clone, uniffi::Record)]
pub struct PubkyKeypair {
    /// Hex-encoded secret key (32 bytes)
    pub secret_key_hex: String,
    /// Public key (z-base-32 encoded)
    pub public_key: String,
}
