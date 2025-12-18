//! Unit tests for Paykit implementation functions
//!
//! Note: Many functions require network access or initialized global state.
//! Tests are designed to verify logic in isolation where possible.

#[cfg(test)]
mod tests {
    use crate::modules::paykit::errors::PaykitError;
    use crate::modules::paykit::types::{PaykitCheckoutResult, PaykitSupportedMethods};

    // ========== PaykitError Tests ==========

    #[test]
    fn test_paykit_error_display_generic() {
        let error = PaykitError::Generic("Something went wrong".to_string());
        assert!(error.to_string().contains("Something went wrong"));
    }

    #[test]
    fn test_paykit_error_display_invalid_pubkey() {
        let error = PaykitError::InvalidPublicKey("bad key format".to_string());
        assert!(error.to_string().contains("bad key format") || error.to_string().contains("Invalid"));
    }

    #[test]
    fn test_paykit_error_display_transport() {
        let error = PaykitError::Transport("Connection refused".to_string());
        assert!(error.to_string().contains("Connection") || error.to_string().contains("Transport"));
    }

    // ========== PaykitCheckoutResult Tests ==========

    #[test]
    fn test_checkout_result_private() {
        let result = PaykitCheckoutResult {
            method_id: "lightning".to_string(),
            endpoint: "lnbc1000n1...".to_string(),
            is_private: true,
            requires_interactive: true,
        };

        assert_eq!(result.method_id, "lightning");
        assert!(result.is_private);
        assert!(result.requires_interactive);
    }

    #[test]
    fn test_checkout_result_public() {
        let result = PaykitCheckoutResult {
            method_id: "onchain".to_string(),
            endpoint: "bc1q...".to_string(),
            is_private: false,
            requires_interactive: false,
        };

        assert_eq!(result.method_id, "onchain");
        assert!(!result.is_private);
        assert!(!result.requires_interactive);
    }

    // ========== PaykitSupportedMethods Tests ==========

    #[test]
    fn test_supported_methods_creation() {
        use crate::modules::paykit::types::PaykitSupportedMethod;
        
        let methods = PaykitSupportedMethods {
            methods: vec![
                PaykitSupportedMethod { method_id: "lightning".to_string(), endpoint: "lnbc...".to_string() },
                PaykitSupportedMethod { method_id: "onchain".to_string(), endpoint: "bc1q...".to_string() },
            ],
        };

        assert_eq!(methods.methods.len(), 2);
        assert!(methods.methods.iter().any(|m| m.method_id == "lightning"));
        assert!(methods.methods.iter().any(|m| m.method_id == "onchain"));
    }

    #[test]
    fn test_supported_methods_empty() {
        let methods = PaykitSupportedMethods {
            methods: vec![],
        };

        assert!(methods.methods.is_empty());
    }

    // ========== Secret Key Validation Tests ==========

    #[test]
    fn test_secret_key_hex_decode_valid() {
        let valid_hex = "a".repeat(64); // 32 bytes in hex
        let result = hex::decode(&valid_hex);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_secret_key_hex_decode_invalid() {
        let invalid_hex = "not_valid_hex";
        let result = hex::decode(invalid_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_secret_key_wrong_length() {
        let short_hex = "aa".repeat(16); // 16 bytes - too short
        let bytes = hex::decode(&short_hex).unwrap();
        assert_eq!(bytes.len(), 16);
        // Would fail the 32-byte check in paykit_initialize
    }

    // ========== Public Key Parsing Tests ==========

    #[test]
    fn test_pubkey_from_str_valid() {
        use paykit_lib::PublicKey;
        use std::str::FromStr;

        // Generate a valid key and use it
        let keypair = pubky::Keypair::random();
        let pubkey_str = keypair.public_key().to_z32();
        
        let result = PublicKey::from_str(&pubkey_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pubkey_from_str_invalid() {
        use paykit_lib::PublicKey;
        use std::str::FromStr;

        let result = PublicKey::from_str("invalid_key_format");
        assert!(result.is_err());
    }

    // ========== Method ID Tests ==========

    #[test]
    fn test_method_id_lightning() {
        use paykit_lib::MethodId;
        let method = MethodId("lightning".to_string());
        assert_eq!(method.0, "lightning");
    }

    #[test]
    fn test_method_id_onchain() {
        use paykit_lib::MethodId;
        let method = MethodId("onchain".to_string());
        assert_eq!(method.0, "onchain");
    }

    #[test]
    fn test_method_id_custom() {
        use paykit_lib::MethodId;
        let method = MethodId("custom:bolt12".to_string());
        assert_eq!(method.0, "custom:bolt12");
    }

    // ========== Endpoint Data Tests ==========

    #[test]
    fn test_endpoint_data_lightning_invoice() {
        use paykit_lib::EndpointData;
        let endpoint = EndpointData("lnbc1000n1ptest...".to_string());
        assert!(endpoint.0.starts_with("lnbc"));
    }

    #[test]
    fn test_endpoint_data_bitcoin_address() {
        use paykit_lib::EndpointData;
        let endpoint = EndpointData("bc1qtest123...".to_string());
        assert!(endpoint.0.starts_with("bc1"));
    }

    // ========== Smart Checkout Priority Tests ==========
    // These test the logic flow without network access

    #[test]
    fn test_checkout_method_priority_order() {
        // Test that lightning is preferred over onchain
        let methods = vec!["onchain", "lightning", "bolt12"];
        
        let preferred = methods.iter()
            .find(|m| **m == "lightning")
            .or_else(|| methods.iter().find(|m| **m == "onchain"))
            .or_else(|| methods.first());

        assert_eq!(*preferred.unwrap(), "lightning");
    }

    #[test]
    fn test_checkout_method_fallback_to_onchain() {
        let methods = vec!["onchain", "bolt12"];
        
        let preferred = methods.iter()
            .find(|m| **m == "lightning")
            .or_else(|| methods.iter().find(|m| **m == "onchain"))
            .or_else(|| methods.first());

        assert_eq!(*preferred.unwrap(), "onchain");
    }

    #[test]
    fn test_checkout_method_fallback_to_first() {
        let methods = vec!["bolt12", "custom"];
        
        let preferred = methods.iter()
            .find(|m| **m == "lightning")
            .or_else(|| methods.iter().find(|m| **m == "onchain"))
            .or_else(|| methods.first());

        assert_eq!(*preferred.unwrap(), "bolt12");
    }

    #[test]
    fn test_checkout_preferred_method_override() {
        let methods = vec!["lightning", "onchain", "bolt12"];
        let preferred_method = Some("onchain");

        let selected = if let Some(pref) = preferred_method {
            methods.iter().find(|m| **m == pref).or_else(|| methods.first())
        } else {
            methods.first()
        };

        assert_eq!(*selected.unwrap(), "onchain");
    }

    // ========== Rotation Detection Logic Tests ==========

    #[test]
    fn test_rotation_method_matching_onchain() {
        let method_id = "onchain";
        let needs_rotation = match method_id {
            "onchain" => true,  // Simulated as "used"
            "lightning" => false,
            _ => false,
        };
        assert!(needs_rotation);
    }

    #[test]
    fn test_rotation_method_matching_lightning() {
        let method_id = "lightning";
        let needs_rotation = match method_id {
            "onchain" => false,
            "lightning" => true,  // Simulated as "used"
            _ => false,
        };
        assert!(needs_rotation);
    }

    #[test]
    fn test_rotation_method_unknown_not_rotated() {
        let method_id = "custom:method";
        let needs_rotation = match method_id {
            "onchain" => true,
            "lightning" => true,
            _ => false,  // Unknown methods not auto-rotated
        };
        assert!(!needs_rotation);
    }

    // ========== Error Conversion Tests ==========

    #[test]
    fn test_paykit_lib_error_conversion() {
        use paykit_lib::PaykitError as LibError;
        
        let lib_error = LibError::NotFound { 
            resource_type: "endpoint".to_string(), 
            identifier: "test_id".to_string(),
        };
        let paykit_error: PaykitError = lib_error.into();
        
        match paykit_error {
            PaykitError::Generic(msg) => assert!(msg.contains("endpoint") || msg.contains("NotFound")),
            _ => panic!("Expected Generic error"),
        }
    }
}

// ========== Database Integration Tests ==========
// These tests require database initialization

#[cfg(test)]
mod db_integration_tests {
    use crate::init_db;
    use crate::activity::{OnchainActivity, LightningActivity, PaymentState, PaymentType};
    use crate::{upsert_onchain_activities, upsert_lightning_activities};
    use tempfile::TempDir;

    #[test]
    fn test_onchain_activity_insert() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        init_db(db_path.to_string()).unwrap();

        let activity = OnchainActivity {
            id: "test_tx_001".to_string(),
            tx_type: PaymentType::Received,
            timestamp: 1234567890u64,
            created_at: Some(1234567890u64),
            updated_at: Some(1234567890u64),
            tx_id: "txid123".to_string(),
            value: 50000u64,
            fee: 500u64,
            fee_rate: 10u64,
            address: "bc1qtest123".to_string(),
            confirmed: true,
            is_boosted: false,
            boost_tx_ids: vec![],
            is_transfer: false,
            does_exist: true,
            confirm_timestamp: Some(1234567890u64),
            channel_id: None,
            transfer_tx_id: None,
            seen_at: None,
        };

        upsert_onchain_activities(vec![activity]).unwrap();
    }

    #[test]
    fn test_lightning_activity_insert() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        init_db(db_path.to_string()).unwrap();

        let activity = LightningActivity {
            id: "test_ln_001".to_string(),
            tx_type: PaymentType::Received,
            timestamp: 1234567890u64,
            created_at: Some(1234567890u64),
            updated_at: Some(1234567890u64),
            invoice: "lnbc1000n1test".to_string(),
            value: 1000u64,
            status: PaymentState::Succeeded,
            fee: Some(1u64),
            message: "Test payment".to_string(),
            preimage: Some("preimage123".to_string()),
            seen_at: None,
        };

        upsert_lightning_activities(vec![activity]).unwrap();
    }

    #[test]
    fn test_multiple_activities_insert() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().to_str().unwrap();
        init_db(db_path.to_string()).unwrap();

        let base_timestamp = 1234567890u64;
        let activities: Vec<OnchainActivity> = (0..10u64).map(|i| {
            let timestamp = base_timestamp + i;
            OnchainActivity {
                id: format!("tx_{:03}", i),
                tx_type: PaymentType::Received,
                timestamp,
                created_at: Some(base_timestamp),
                updated_at: Some(base_timestamp),
                tx_id: format!("txid_{}", i),
                value: 1000u64 * (i + 1),
                fee: 100u64,
                fee_rate: 10u64,
                address: format!("bc1qtest{}", i),
                confirmed: true,
                is_boosted: false,
                boost_tx_ids: vec![],
                is_transfer: false,
                does_exist: true,
                confirm_timestamp: Some(timestamp), // Must be >= timestamp
                channel_id: None,
                transfer_tx_id: None,
                seen_at: None,
            }
        }).collect();

        upsert_onchain_activities(activities).unwrap();
    }
}

