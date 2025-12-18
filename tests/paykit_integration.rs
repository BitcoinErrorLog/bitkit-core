//! Integration tests for Paykit payment flows
//!
//! These tests verify complete workflows across multiple components.
//! Run with: cargo test --test paykit_integration

use bitkitcore::activity::{Activity, LightningActivity, OnchainActivity, PaymentState, PaymentType};
use tempfile::TempDir;

/// Create a temporary database for testing
fn create_test_db() -> (String, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    (db_path.to_str().unwrap().to_string(), temp_dir)
}

/// Create a test onchain activity
fn create_test_onchain_activity(id: &str, address: &str, value: u64) -> OnchainActivity {
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
fn create_test_lightning_activity(id: &str, invoice: &str, value: u64) -> LightningActivity {
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

/// Test 1: Activity storage and retrieval flow
#[test]
fn test_activity_storage_flow() {
    let (db_path, _temp) = create_test_db();
    bitkitcore::init_db(db_path).unwrap();

    // Store onchain activity
    let onchain = create_test_onchain_activity("tx_001", "bc1qtest123", 50000);
    bitkitcore::upsert_onchain_activities(vec![onchain]).unwrap();

    // Store lightning activity
    let lightning = create_test_lightning_activity("ln_001", "lnbc1000n1test", 1000);
    bitkitcore::upsert_lightning_activities(vec![lightning]).unwrap();

    // Retrieve all activities
    let activities = bitkitcore::get_activities(None, None, None, None, None, None, None, None).unwrap();
    assert!(activities.len() >= 2);
}

/// Test 2: Multiple activity batch processing
#[test]
fn test_batch_activity_processing() {
    let (db_path, _temp) = create_test_db();
    bitkitcore::init_db(db_path).unwrap();

    // Create batch of onchain activities
    let onchain_batch: Vec<_> = (0..5)
        .map(|i| create_test_onchain_activity(
            &format!("batch_tx_{}", i),
            &format!("bc1qbatch{}", i),
            1000 * (i + 1) as u64,
        ))
        .collect();

    bitkitcore::upsert_onchain_activities(onchain_batch).unwrap();

    // Create batch of lightning activities  
    let lightning_batch: Vec<_> = (0..5)
        .map(|i| create_test_lightning_activity(
            &format!("batch_ln_{}", i),
            &format!("lnbc{}n1batch", i),
            100 * (i + 1) as u64,
        ))
        .collect();

    bitkitcore::upsert_lightning_activities(lightning_batch).unwrap();

    let activities = bitkitcore::get_activities(None, None, None, None, None, None, None, None).unwrap();
    assert!(activities.len() >= 10);
}

/// Test 3: Activity update flow
#[test]
fn test_activity_update_flow() {
    let (db_path, _temp) = create_test_db();
    bitkitcore::init_db(db_path).unwrap();

    // Create initial activity
    let mut onchain = create_test_onchain_activity("update_tx", "bc1qupdate", 10000);
    bitkitcore::upsert_onchain_activities(vec![onchain.clone()]).unwrap();

    // Update the activity
    onchain.value = 20000;
    onchain.confirmed = true;
    bitkitcore::upsert_onchain_activities(vec![onchain]).unwrap();

    // Activity should still exist (update, not duplicate)
    let activities = bitkitcore::get_activities(None, None, None, None, None, None, None, None).unwrap();
    let updated = activities.iter().find(|a| {
        matches!(a, Activity::Onchain(o) if o.id == "update_tx")
    });
    assert!(updated.is_some());
}

/// Test 4: Mixed activities with different types
#[test]
fn test_mixed_activity_types() {
    let (db_path, _temp) = create_test_db();
    bitkitcore::init_db(db_path).unwrap();

    // Store various payment types
    let received = create_test_onchain_activity("received_tx", "bc1qreceived", 100000);
    bitkitcore::upsert_onchain_activities(vec![received]).unwrap();

    let mut sent = create_test_onchain_activity("sent_tx", "bc1qsent", 50000);
    sent.tx_type = PaymentType::Sent;
    bitkitcore::upsert_onchain_activities(vec![sent]).unwrap();

    let ln_received = create_test_lightning_activity("ln_received", "lnbc_received", 5000);
    bitkitcore::upsert_lightning_activities(vec![ln_received]).unwrap();

    let mut ln_sent = create_test_lightning_activity("ln_sent", "lnbc_sent", 2500);
    ln_sent.tx_type = PaymentType::Sent;
    bitkitcore::upsert_lightning_activities(vec![ln_sent]).unwrap();

    let activities = bitkitcore::get_activities(None, None, None, None, None, None, None, None).unwrap();
    assert!(activities.len() >= 4);

    // Verify different types are stored
    let has_received = activities.iter().any(|a| matches!(a, 
        Activity::Onchain(o) if o.tx_type == PaymentType::Received && o.id == "received_tx"
    ));
    let has_sent = activities.iter().any(|a| matches!(a,
        Activity::Onchain(o) if o.tx_type == PaymentType::Sent && o.id == "sent_tx"
    ));
    
    assert!(has_received);
    assert!(has_sent);
}

/// Test 5: Large batch activity processing
#[test]
fn test_large_batch_processing() {
    let (db_path, _temp) = create_test_db();
    bitkitcore::init_db(db_path).unwrap();

    // Create large batch (100 activities)
    let large_batch: Vec<_> = (0..100)
        .map(|i| create_test_onchain_activity(
            &format!("large_batch_{}", i),
            &format!("bc1qlarge{}", i),
            (i + 1) as u64 * 100,
        ))
        .collect();

    bitkitcore::upsert_onchain_activities(large_batch).unwrap();

    let activities = bitkitcore::get_activities(None, None, None, None, None, None, None, None).unwrap();
    assert!(activities.len() >= 100);
}

/// Test 6: Activity with all fields populated
#[test]
fn test_activity_all_fields() {
    let (db_path, _temp) = create_test_db();
    bitkitcore::init_db(db_path).unwrap();

    let timestamp = 1700000000u64;
    let full_activity = OnchainActivity {
        id: "full_tx".to_string(),
        tx_type: PaymentType::Received,
        timestamp,
        created_at: Some(timestamp),
        updated_at: Some(timestamp + 1000),
        tx_id: "full_txid_hash".to_string(),
        value: 1000000u64,
        fee: 1500u64,
        fee_rate: 15u64,
        address: "bc1qfulladdresstest".to_string(),
        confirmed: true,
        is_boosted: true,
        boost_tx_ids: vec!["boost_tx_1".to_string(), "boost_tx_2".to_string()],
        is_transfer: false,
        does_exist: true,
        confirm_timestamp: Some(timestamp + 600),
        channel_id: Some("channel_123".to_string()),
        transfer_tx_id: None,
        seen_at: Some(timestamp - 100),
    };

    bitkitcore::upsert_onchain_activities(vec![full_activity]).unwrap();

    let activities = bitkitcore::get_activities(None, None, None, None, None, None, None, None).unwrap();
    let found = activities.iter().find(|a| {
        matches!(a, Activity::Onchain(o) if o.id == "full_tx")
    });
    assert!(found.is_some());
}
