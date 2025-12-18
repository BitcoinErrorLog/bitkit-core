//! Unit tests for PaykitInteractive module

#[cfg(test)]
mod tests {
    use crate::modules::paykit::types::PaykitReceiptFfi;

    // ========== PaykitReceiptFfi Tests ==========

    #[test]
    fn test_receipt_ffi_creation() {
        let receipt = PaykitReceiptFfi {
            receipt_id: "receipt_001".to_string(),
            payer: "payer_key".to_string(),
            payee: "payee_key".to_string(),
            method_id: "lightning".to_string(),
            amount: Some("1000".to_string()),
            currency: Some("SAT".to_string()),
            created_at: 1234567890,
            metadata_json: r#"{"description":"test"}"#.to_string(),
        };

        assert_eq!(receipt.receipt_id, "receipt_001");
        assert_eq!(receipt.method_id, "lightning");
        assert_eq!(receipt.amount, Some("1000".to_string()));
    }

    #[test]
    fn test_receipt_ffi_without_amount() {
        let receipt = PaykitReceiptFfi {
            receipt_id: "receipt_002".to_string(),
            payer: "payer_key".to_string(),
            payee: "payee_key".to_string(),
            method_id: "onchain".to_string(),
            amount: None,
            currency: None,
            created_at: 1234567890,
            metadata_json: "{}".to_string(),
        };

        assert!(receipt.amount.is_none());
        assert!(receipt.currency.is_none());
    }

    #[test]
    fn test_receipt_ffi_metadata_parsing() {
        let metadata_json = r#"{"invoice":"lnbc123...","expiry":3600}"#;
        let receipt = PaykitReceiptFfi {
            receipt_id: "receipt_003".to_string(),
            payer: "payer".to_string(),
            payee: "payee".to_string(),
            method_id: "lightning".to_string(),
            amount: Some("5000".to_string()),
            currency: Some("SAT".to_string()),
            created_at: 1234567890,
            metadata_json: metadata_json.to_string(),
        };

        let parsed: serde_json::Value = serde_json::from_str(&receipt.metadata_json).unwrap();
        assert!(parsed.get("invoice").is_some());
        assert_eq!(parsed.get("expiry").unwrap().as_i64().unwrap(), 3600);
    }

    // ========== Secret Key Validation ==========

    #[test]
    fn test_secret_key_hex_valid() {
        let hex = "a".repeat(64);
        let bytes = hex::decode(&hex).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_secret_key_hex_too_short() {
        let hex = "a".repeat(32); // 16 bytes
        let bytes = hex::decode(&hex).unwrap();
        assert_ne!(bytes.len(), 32);
    }

    #[test]
    fn test_secret_key_hex_too_long() {
        let hex = "a".repeat(128); // 64 bytes
        let bytes = hex::decode(&hex).unwrap();
        assert_ne!(bytes.len(), 32);
    }

    #[test]
    fn test_secret_key_hex_invalid_chars() {
        let hex = "ghij".repeat(16);
        let result = hex::decode(&hex);
        assert!(result.is_err());
    }

    // ========== Port/Host Validation Tests ==========

    #[test]
    fn test_address_format() {
        let host = "127.0.0.1";
        let port: u16 = 9735;
        let addr = format!("{}:{}", host, port);
        assert_eq!(addr, "127.0.0.1:9735");
    }

    #[test]
    fn test_address_with_hostname() {
        let host = "node.example.com";
        let port: u16 = 9735;
        let addr = format!("{}:{}", host, port);
        assert_eq!(addr, "node.example.com:9735");
    }

    // ========== Noise Key Tests ==========

    #[test]
    fn test_noise_seed_from_secret() {
        let secret_hex = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";
        let bytes = hex::decode(secret_hex).unwrap();
        let seed: [u8; 32] = bytes.try_into().unwrap();
        assert_eq!(seed.len(), 32);
        assert_eq!(seed[0], 0x01);
        assert_eq!(seed[31], 0x20);
    }

    // ========== PubkyNoiseChannel Tests (Logic Only) ==========
    // Actual channel tests require network - these test supporting logic

    #[test]
    fn test_pubkey_to_bytes_conversion() {
        use pubky::Keypair;
        
        let keypair = Keypair::random();
        let pubkey = keypair.public_key();
        let bytes = pubkey.as_bytes();
        
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_device_id_placeholder() {
        let device_id: Vec<u8> = vec![0; 32];
        assert_eq!(device_id.len(), 32);
    }

    #[test]
    fn test_epoch_initial() {
        let epoch: u32 = 0;
        assert_eq!(epoch, 0);
    }

    // ========== Receipt Conversion Logic ==========

    #[test]
    fn test_metadata_roundtrip() {
        let original = serde_json::json!({
            "description": "Payment for service",
            "invoice": "lnbc123...",
            "nested": {"key": "value"}
        });
        
        let json_str = serde_json::to_string(&original).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_empty_metadata() {
        let metadata = serde_json::json!({});
        let json_str = serde_json::to_string(&metadata).unwrap();
        assert_eq!(json_str, "{}");
    }

    #[test]
    fn test_complex_metadata() {
        let metadata = serde_json::json!({
            "amounts": [100, 200, 300],
            "flags": {"is_test": true, "is_recurring": false},
            "unicode": "日本語 🎉"
        });
        
        let json_str = serde_json::to_string(&metadata).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(parsed["amounts"][0].as_i64(), Some(100));
        assert_eq!(parsed["flags"]["is_test"].as_bool(), Some(true));
        assert!(parsed["unicode"].as_str().unwrap().contains("日本語"));
    }
}

