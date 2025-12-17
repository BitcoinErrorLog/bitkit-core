//! Unit tests for pubky_sdk FFI module

#[cfg(test)]
mod tests {
    use crate::modules::pubky_sdk::functions::*;
    use crate::modules::pubky_sdk::types::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = pubky_generate_keypair();
        
        // Secret key should be 64 hex chars (32 bytes)
        assert_eq!(keypair.secret_key_hex.len(), 64);
        
        // Public key should be a valid z-base-32 string (52 chars typically)
        assert!(!keypair.public_key.is_empty());
        
        // Each generation should produce unique keys
        let keypair2 = pubky_generate_keypair();
        assert_ne!(keypair.secret_key_hex, keypair2.secret_key_hex);
        assert_ne!(keypair.public_key, keypair2.public_key);
    }

    #[test]
    fn test_public_key_from_secret_valid() {
        // Generate a keypair first
        let keypair = pubky_generate_keypair();
        
        // Derive public key from the secret - should match original
        let derived = pubky_public_key_from_secret(keypair.secret_key_hex.clone()).unwrap();
        assert_eq!(derived, keypair.public_key);
    }

    #[test]
    fn test_public_key_from_secret_invalid_hex() {
        let result = pubky_public_key_from_secret("not_valid_hex".to_string());
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        match err {
            crate::modules::pubky_sdk::errors::PubkyError::InvalidInput { message } => {
                assert!(message.contains("Invalid hex"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_public_key_from_secret_wrong_length() {
        // Valid hex but wrong length (only 16 bytes)
        let short_key = "00112233445566778899aabbccddeeff";
        let result = pubky_public_key_from_secret(short_key.to_string());
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            crate::modules::pubky_sdk::errors::PubkyError::InvalidInput { message } => {
                assert!(message.contains("32 bytes"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_public_key_derivation_deterministic() {
        // Same secret key should always produce same public key
        let secret = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        
        let pubkey1 = pubky_public_key_from_secret(secret.to_string()).unwrap();
        let pubkey2 = pubky_public_key_from_secret(secret.to_string()).unwrap();
        
        assert_eq!(pubkey1, pubkey2);
    }

    #[test]
    fn test_keypair_struct_fields() {
        let keypair = PubkyKeypair {
            secret_key_hex: "deadbeef".repeat(8),
            public_key: "test_pubkey".to_string(),
        };
        
        assert_eq!(keypair.secret_key_hex.len(), 64);
        assert_eq!(keypair.public_key, "test_pubkey");
    }

    #[test]
    fn test_session_info_struct() {
        let session_info = PubkySessionInfo {
            pubkey: "test_pubkey".to_string(),
            capabilities: vec!["read".to_string(), "write".to_string()],
            created_at: 1234567890,
        };
        
        assert_eq!(session_info.pubkey, "test_pubkey");
        assert_eq!(session_info.capabilities.len(), 2);
        assert_eq!(session_info.created_at, 1234567890);
    }

    #[test]
    fn test_profile_struct() {
        let profile = PubkyProfile {
            name: Some("Alice".to_string()),
            bio: Some("Hello world".to_string()),
            image: Some("https://example.com/avatar.png".to_string()),
            links: vec!["https://twitter.com/alice".to_string()],
            status: Some("Online".to_string()),
        };
        
        assert_eq!(profile.name, Some("Alice".to_string()));
        assert_eq!(profile.bio, Some("Hello world".to_string()));
        assert_eq!(profile.image, Some("https://example.com/avatar.png".to_string()));
        assert_eq!(profile.links.len(), 1);
        assert_eq!(profile.status, Some("Online".to_string()));
    }

    #[test]
    fn test_list_item_struct() {
        let file_item = PubkyListItem {
            name: "document.txt".to_string(),
            path: "/pub/documents/document.txt".to_string(),
            is_directory: false,
        };
        
        let dir_item = PubkyListItem {
            name: "photos".to_string(),
            path: "/pub/photos/".to_string(),
            is_directory: true,
        };
        
        assert!(!file_item.is_directory);
        assert!(dir_item.is_directory);
    }

    #[test]
    fn test_signup_options_struct() {
        let options_with_token = PubkySignupOptions {
            signup_token: Some("abc123".to_string()),
        };
        
        let options_without_token = PubkySignupOptions {
            signup_token: None,
        };
        
        assert!(options_with_token.signup_token.is_some());
        assert!(options_without_token.signup_token.is_none());
    }
}

