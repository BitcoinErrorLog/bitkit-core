use rusqlite::OptionalExtension;
use crate::modules::paykit::errors::PaykitError;
use crate::modules::paykit::types::{PaykitCheckoutResult, PaykitSupportedMethods};
use once_cell::sync::OnceCell;
use paykit_lib::{
    EndpointData, MethodId, PubkyAuthenticatedTransport, PubkyUnauthenticatedTransport,
};
use pubky::{Keypair, Pubky, PublicKey as PubkyId};
use std::str::FromStr;

static PAYKIT_SESSION: OnceCell<PubkyAuthenticatedTransport> = OnceCell::new();
static PAYKIT_READER: OnceCell<PubkyUnauthenticatedTransport> = OnceCell::new();

/// Initialize the Paykit authenticated session.
///
/// * `secret_key_hex`: Hex-encoded secret key (32 bytes).
/// * `homeserver_pubkey`: The public key of the user's homeserver (z-base-32 string).
pub async fn paykit_initialize(
    secret_key_hex: String,
    homeserver_pubkey: String,
) -> Result<(), PaykitError> {
    let bytes = hex::decode(&secret_key_hex)
        .map_err(|e| PaykitError::Generic(format!("Invalid secret key hex: {}", e)))?;

    if bytes.len() != 32 {
        return Err(PaykitError::Generic("Secret key must be 32 bytes".to_string()));
    }
    
    let mut secret_arr = [0u8; 32];
    secret_arr.copy_from_slice(&bytes);
    
    let keypair = Keypair::from_secret_key(&secret_arr);

    let homeserver = PubkyId::try_from(homeserver_pubkey.as_str())
        .map_err(|e| PaykitError::InvalidPublicKey(e.to_string()))?;

    let sdk = Pubky::new()
        .map_err(|e| PaykitError::Transport(e.to_string()))?;

    let signer = sdk.signer(keypair);

    // Authenticate with the homeserver to create a session
    let session = signer
        .signup(&homeserver, None)
        .await
        .map_err(|e| PaykitError::Transport(format!("Failed to create session: {}", e)))?;

    let transport = PubkyAuthenticatedTransport::new(session);

    // Set global session (ignore error if already set)
    let _ = PAYKIT_SESSION.set(transport);

    Ok(())
}

pub async fn paykit_ensure_reader() -> Result<&'static PubkyUnauthenticatedTransport, PaykitError> {
    if let Some(reader) = PAYKIT_READER.get() {
        return Ok(reader);
    }

    let sdk = Pubky::new()
        .map_err(|e| PaykitError::Transport(e.to_string()))?;
    let reader = PubkyUnauthenticatedTransport::new(sdk.public_storage());

    PAYKIT_READER
        .set(reader)
        .map_err(|_| PaykitError::Generic("Failed to set global reader".into()))?;
    Ok(PAYKIT_READER.get().unwrap())
}

pub async fn paykit_get_session() -> Result<&'static PubkyAuthenticatedTransport, PaykitError> {
    PAYKIT_SESSION.get().ok_or(PaykitError::Generic(
        "Paykit session not initialized. Call paykit_initialize first.".into(),
    ))
}

pub async fn paykit_set_endpoint(method_id: String, endpoint: String) -> Result<(), PaykitError> {
    let session = paykit_get_session().await?;
    paykit_lib::set_payment_endpoint(session, MethodId(method_id), EndpointData(endpoint))
        .await
        .map_err(PaykitError::from)
}

pub async fn paykit_remove_endpoint(method_id: String) -> Result<(), PaykitError> {
    let session = paykit_get_session().await?;
    paykit_lib::remove_payment_endpoint(session, MethodId(method_id))
        .await
        .map_err(PaykitError::from)
}

pub async fn paykit_get_supported_methods_for_key(
    pubkey: String,
) -> Result<PaykitSupportedMethods, PaykitError> {
    let reader = paykit_ensure_reader().await?;
    let pk = paykit_lib::PublicKey::from_str(&pubkey)
        .map_err(|e| PaykitError::InvalidPublicKey(e.to_string()))?;

    let methods = paykit_lib::get_payment_list(reader, &pk).await?;
    Ok(methods.into())
}

pub async fn paykit_get_endpoint_for_key_and_method(
    pubkey: String,
    method_id: String,
) -> Result<Option<String>, PaykitError> {
    let reader = paykit_ensure_reader().await?;
    let pk = paykit_lib::PublicKey::from_str(&pubkey)
        .map_err(|e| PaykitError::InvalidPublicKey(e.to_string()))?;

    let endpoint = paykit_lib::get_payment_endpoint(reader, &pk, &MethodId(method_id)).await?;
    Ok(endpoint.map(|e| e.0))
}

/// Checks which payment methods have been used (received funds) and need rotation.
/// Returns a list of method IDs (e.g. "onchain", "lightning") that should be rotated.
pub async fn paykit_check_rotation_needed(pubkey: String) -> Result<Vec<String>, PaykitError> {
    let reader = paykit_ensure_reader().await?;
    let pk = paykit_lib::PublicKey::from_str(&pubkey)
        .map_err(|e| PaykitError::InvalidPublicKey(e.to_string()))?;

    let methods = paykit_lib::get_payment_list(reader, &pk).await?;

    let db_guard = crate::get_activity_db().map_err(|e| PaykitError::Generic(e.to_string()))?;
    let activity_db = db_guard.activity_db.as_ref().ok_or(PaykitError::Generic(
        "Activity DB not initialized".to_string(),
    ))?;

    let mut to_rotate = Vec::new();

    for (method_id, data) in methods.entries {
        let used = match method_id.0.as_str() {
            "onchain" => activity_db
                .has_onchain_received(&data.0)
                .map_err(|e| PaykitError::Generic(e.to_string()))?,
            "lightning" => activity_db
                .has_lightning_paid(&data.0)
                .map_err(|e| PaykitError::Generic(e.to_string()))?,
            _ => false, // Other methods or custom ones are not auto-rotated by this logic
        };

        if used {
            to_rotate.push(method_id.0);
        }
    }

    Ok(to_rotate)
}

/// Smart checkout flow: tries private offer first, then falls back to public directory.
///
/// Returns the best available payment method for the given peer.
///
/// # Arguments
/// * `pubkey` - The peer's public key (Pubky ID)
/// * `preferred_method` - Optional preferred method ID (e.g., "lightning", "onchain")
///
/// # Returns
/// * `PaykitCheckoutResult` containing the method to use and whether it's private
pub async fn paykit_smart_checkout(
    pubkey: String,
    preferred_method: Option<String>,
) -> Result<PaykitCheckoutResult, PaykitError> {
    let pk = paykit_lib::PublicKey::from_str(&pubkey)
        .map_err(|e| PaykitError::InvalidPublicKey(e.to_string()))?;

    // Step 1: Check for private offer (if we have a storage implementation)
    if let Ok(private_endpoint) = check_private_offer(&pk).await {
        if let Some(endpoint) = private_endpoint {
            return Ok(PaykitCheckoutResult {
                method_id: endpoint.method_id,
                endpoint: endpoint.endpoint,
                is_private: true,
                requires_interactive: true,
            });
        }
    }

    // Step 2: Fall back to public directory
    let reader = paykit_ensure_reader().await?;
    let methods = paykit_lib::get_payment_list(reader, &pk).await?;

    if methods.entries.is_empty() {
        return Err(PaykitError::Generic(
            "No payment methods available for this peer".to_string(),
        ));
    }

    // Step 3: Select best method based on preference
    let entries: Vec<_> = methods.entries.into_iter().collect();
    
    let selected = if let Some(preferred) = &preferred_method {
        // Try to find preferred method
        entries.iter()
            .find(|(id, _)| &id.0 == preferred)
            .or_else(|| entries.first())
            .ok_or_else(|| PaykitError::Generic("No methods available".to_string()))?
    } else {
        // Default preference: lightning > onchain > others
        entries.iter()
            .find(|(id, _)| id.0 == "lightning")
            .or_else(|| entries.iter().find(|(id, _)| id.0 == "onchain"))
            .or_else(|| entries.first())
            .ok_or_else(|| PaykitError::Generic("No methods available".to_string()))?
    };

    Ok(PaykitCheckoutResult {
        method_id: selected.0.0.clone(),
        endpoint: selected.1.0.clone(),
        is_private: false,
        requires_interactive: false,
    })
}

/// Helper function to check if we have a private offer for this peer
async fn check_private_offer(
    peer: &paykit_lib::PublicKey,
) -> Result<Option<PrivateEndpoint>, PaykitError> {
    let db_guard = crate::get_activity_db().map_err(|e| PaykitError::Generic(e.to_string()))?;
    let activity_db = db_guard.activity_db.as_ref().ok_or(PaykitError::Generic(
        "Activity DB not initialized".to_string(),
    ))?;

    // Format must match BitkitPaykitStorage implementation
    let peer_str = format!("{:?}", peer);

    // Check for lightning first (preferred)
    // We handle the case where the table might not exist yet (returns error -> treat as None)
    let lightning = activity_db
        .conn
        .query_row(
            "SELECT method_id, endpoint FROM paykit_private_endpoints 
             WHERE peer = ?1 AND method_id = 'lightning'",
            [&peer_str],
            |row| {
                Ok(PrivateEndpoint {
                    method_id: row.get(0)?,
                    endpoint: row.get(1)?,
                })
            },
        )
        .optional()
        .unwrap_or(None); // If table missing or other error, ignore

    if let Some(endpoint) = lightning {
        return Ok(Some(endpoint));
    }

    // Check for onchain
    let onchain = activity_db
        .conn
        .query_row(
            "SELECT method_id, endpoint FROM paykit_private_endpoints 
             WHERE peer = ?1 AND method_id = 'onchain'",
            [&peer_str],
            |row| {
                Ok(PrivateEndpoint {
                    method_id: row.get(0)?,
                    endpoint: row.get(1)?,
                })
            },
        )
        .optional()
        .unwrap_or(None);

    Ok(onchain)
}

#[derive(Debug, Clone)]
struct PrivateEndpoint {
    method_id: String,
    endpoint: String,
}
