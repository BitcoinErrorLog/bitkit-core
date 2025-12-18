//! Integration tests for Paykit payment flows
//!
//! These tests verify complete workflows across multiple components.

use super::test_utils::*;

/// Test 1: Activity storage and retrieval flow
/// 
/// This tests the complete flow of:
/// 1. Initialize database
/// 2. Store onchain and lightning activities
/// 3. Verify activities can be retrieved
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
    let activities = bitkitcore::get_all_activities().unwrap();
    assert!(activities.len() >= 2);
}

/// Test 2: Multiple activity batch processing
///
/// Tests batch insert of multiple activities
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

    let activities = bitkitcore::get_all_activities().unwrap();
    assert!(activities.len() >= 10);
}

/// Test 3: Activity update flow
///
/// Tests that updating an activity works correctly
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
    let activities = bitkitcore::get_all_activities().unwrap();
    let updated = activities.iter().find(|a| {
        matches!(a, bitkitcore::activity::Activity::Onchain(o) if o.id == "update_tx")
    });
    assert!(updated.is_some());
}

/// Test 4: Paykit storage integration
///
/// Tests PaykitStorage operations through the BitkitPaykitStorage implementation
#[tokio::test]
async fn test_paykit_storage_integration() {
    use bitkitcore::modules::paykit::storage::BitkitPaykitStorage;
    use paykit_interactive::PaykitStorage;
    use paykit_lib::MethodId;
    use pubky::Keypair;

    let (db_path, _temp) = create_test_db();
    let storage = BitkitPaykitStorage::new(&db_path).unwrap();
    let peer = Keypair::random().public_key();

    // Save private endpoint
    storage.save_private_endpoint(
        &peer,
        &MethodId("lightning".to_string()),
        "lnbc1000n1private_offer",
    ).await.unwrap();

    // Retrieve private endpoint
    let endpoint = storage.get_private_endpoint(
        &peer,
        &MethodId("lightning".to_string()),
    ).await.unwrap();

    assert_eq!(endpoint, Some("lnbc1000n1private_offer".to_string()));

    // List endpoints for peer
    let endpoints = storage.list_private_endpoints_for_peer(&peer).await.unwrap();
    assert_eq!(endpoints.len(), 1);

    // Remove endpoint
    storage.remove_private_endpoint(
        &peer,
        &MethodId("lightning".to_string()),
    ).await.unwrap();

    let after_remove = storage.get_private_endpoint(
        &peer,
        &MethodId("lightning".to_string()),
    ).await.unwrap();
    assert!(after_remove.is_none());
}

/// Test 5: Receipt generation flow
///
/// Tests the BitkitReceiptGenerator for different payment methods
#[tokio::test]
async fn test_receipt_generation_flow() {
    use bitkitcore::modules::paykit::receipt_generator::BitkitReceiptGenerator;
    use paykit_interactive::{PaykitReceipt, ReceiptGenerator};
    use paykit_lib::MethodId;
    use pubky::Keypair;
    use serde_json::json;

    let generator = BitkitReceiptGenerator::new();
    
    // Test lightning receipt
    let lightning_request = PaykitReceipt::new(
        "receipt_ln_001".to_string(),
        Keypair::random().public_key(),
        Keypair::random().public_key(),
        MethodId("lightning".to_string()),
        Some("5000".to_string()),
        Some("SAT".to_string()),
        json!({"description": "Test lightning payment"}),
    );

    let lightning_result = generator.generate_receipt(&lightning_request).await.unwrap();
    assert!(lightning_result.metadata.get("invoice").is_some());
    assert!(lightning_result.metadata.get("generated_at").is_some());

    // Test onchain receipt
    let onchain_request = PaykitReceipt::new(
        "receipt_btc_001".to_string(),
        Keypair::random().public_key(),
        Keypair::random().public_key(),
        MethodId("onchain".to_string()),
        Some("100000".to_string()),
        Some("SAT".to_string()),
        json!({}),
    );

    let onchain_result = generator.generate_receipt(&onchain_request).await.unwrap();
    assert!(onchain_result.metadata.get("address").is_some());
    assert!(onchain_result.metadata.get("generated_at").is_some());
}

/// Test 6: Error handling across components
///
/// Tests that errors are properly propagated between components
#[tokio::test]
async fn test_error_propagation() {
    use bitkitcore::modules::paykit::receipt_generator::BitkitReceiptGenerator;
    use paykit_interactive::{PaykitReceipt, ReceiptGenerator};
    use paykit_lib::MethodId;
    use pubky::Keypair;
    use serde_json::json;

    let generator = BitkitReceiptGenerator::new();

    // Test unsupported method error
    let unsupported_request = PaykitReceipt::new(
        "receipt_err_001".to_string(),
        Keypair::random().public_key(),
        Keypair::random().public_key(),
        MethodId("unsupported_method".to_string()),
        Some("1000".to_string()),
        Some("SAT".to_string()),
        json!({}),
    );

    let result = generator.generate_receipt(&unsupported_request).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported"));

    // Test zero amount error
    let zero_amount_request = PaykitReceipt::new(
        "receipt_err_002".to_string(),
        Keypair::random().public_key(),
        Keypair::random().public_key(),
        MethodId("lightning".to_string()),
        Some("0".to_string()),
        Some("SAT".to_string()),
        json!({}),
    );

    let result = generator.generate_receipt(&zero_amount_request).await;
    assert!(result.is_err());
}

