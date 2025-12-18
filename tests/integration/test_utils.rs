//! Test utilities for integration tests

use bitkitcore::activity::{LightningActivity, OnchainActivity, PaymentState, PaymentType};
use tempfile::TempDir;

/// Create a temporary database for testing
pub fn create_test_db() -> (String, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    (db_path.to_str().unwrap().to_string(), temp_dir)
}

/// Create a test onchain activity
pub fn create_test_onchain_activity(id: &str, address: &str, value: u64) -> OnchainActivity {
    let timestamp = 1700000000u64;
    OnchainActivity {
        id: id.to_string(),
        tx_type: PaymentType::Received,
        timestamp,
        created_at: Some(timestamp),
        updated_at: Some(timestamp),
        tx_id: format!("txid_{}", id),
        value,
        fee: 500u64,
        fee_rate: 10u64,
        address: address.to_string(),
        confirmed: true,
        is_boosted: false,
        boost_tx_ids: vec![],
        is_transfer: false,
        does_exist: true,
        confirm_timestamp: Some(timestamp),
        channel_id: None,
        transfer_tx_id: None,
        seen_at: None,
    }
}

/// Create a test lightning activity
pub fn create_test_lightning_activity(id: &str, invoice: &str, value: u64) -> LightningActivity {
    let timestamp = 1700000000u64;
    LightningActivity {
        id: id.to_string(),
        tx_type: PaymentType::Received,
        timestamp,
        created_at: Some(timestamp),
        updated_at: Some(timestamp),
        invoice: invoice.to_string(),
        value,
        status: PaymentState::Succeeded,
        fee: Some(1u64),
        message: "Test payment".to_string(),
        preimage: Some(format!("preimage_{}", id)),
        seen_at: None,
    }
}

/// Generate a valid test secret key (32 bytes hex)
pub fn test_secret_key() -> String {
    "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20".to_string()
}

/// Generate a random pubkey string for testing
pub fn random_test_pubkey() -> String {
    use pubky::Keypair;
    Keypair::random().public_key().to_z32()
}

