//! FFI-exported functions for pubky SDK operations

use super::errors::PubkyError;
use super::types::{PubkyKeypair, PubkyListItem, PubkyProfile, PubkySessionInfo, PubkySignupOptions};
use once_cell::sync::OnceCell;
use pubky::{Keypair, Pubky, PubkySession, PublicKey};
use std::collections::HashMap;
use tokio::sync::RwLock;

// Global state for SDK and sessions
static PUBKY_SDK: OnceCell<Pubky> = OnceCell::new();
static SESSIONS: OnceCell<RwLock<HashMap<String, PubkySession>>> = OnceCell::new();

fn get_sdk() -> Result<&'static Pubky, PubkyError> {
    PUBKY_SDK.get().ok_or_else(|| PubkyError::Build {
        message: "Pubky SDK not initialized. Call pubky_initialize first.".to_string(),
    })
}

fn get_sessions() -> &'static RwLock<HashMap<String, PubkySession>> {
    SESSIONS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Initialize the Pubky SDK
#[uniffi::export]
pub fn pubky_initialize() -> Result<(), PubkyError> {
    let sdk = Pubky::new()?;
    let _ = PUBKY_SDK.set(sdk);
    Ok(())
}

/// Initialize the Pubky SDK for testnet
#[uniffi::export]
pub fn pubky_initialize_testnet() -> Result<(), PubkyError> {
    let sdk = Pubky::testnet()?;
    let _ = PUBKY_SDK.set(sdk);
    Ok(())
}

/// Sign in with a secret key (hex-encoded, 32 bytes)
#[uniffi::export]
pub async fn pubky_signin(secret_key_hex: String) -> Result<PubkySessionInfo, PubkyError> {
    let sdk = get_sdk()?;
    
    let secret_bytes = hex::decode(&secret_key_hex)
        .map_err(|e| PubkyError::InvalidInput { message: format!("Invalid hex: {}", e) })?;
    
    if secret_bytes.len() != 32 {
        return Err(PubkyError::InvalidInput {
            message: "Secret key must be 32 bytes".to_string(),
        });
    }
    
    let mut secret_arr = [0u8; 32];
    secret_arr.copy_from_slice(&secret_bytes);
    
    let keypair = Keypair::from_secret_key(&secret_arr);
    let signer = sdk.signer(keypair);
    
    let session = signer.signin().await?;
    
    let info = session.info();
    let session_info = PubkySessionInfo {
        pubkey: info.public_key().to_string(),
        capabilities: info.capabilities().iter().map(|c| c.to_string()).collect(),
        created_at: info.created_at(),
    };
    
    // Store session for later use
    let mut sessions = get_sessions().write().await;
    sessions.insert(session_info.pubkey.clone(), session);
    
    Ok(session_info)
}

/// Sign up with a secret key and homeserver
#[uniffi::export]
pub async fn pubky_signup(
    secret_key_hex: String,
    homeserver_pubkey: String,
    options: Option<PubkySignupOptions>,
) -> Result<PubkySessionInfo, PubkyError> {
    let sdk = get_sdk()?;
    
    let secret_bytes = hex::decode(&secret_key_hex)
        .map_err(|e| PubkyError::InvalidInput { message: format!("Invalid hex: {}", e) })?;
    
    if secret_bytes.len() != 32 {
        return Err(PubkyError::InvalidInput {
            message: "Secret key must be 32 bytes".to_string(),
        });
    }
    
    let mut secret_arr = [0u8; 32];
    secret_arr.copy_from_slice(&secret_bytes);
    
    let keypair = Keypair::from_secret_key(&secret_arr);
    let homeserver = PublicKey::try_from(homeserver_pubkey.as_str())
        .map_err(|e| PubkyError::InvalidInput { message: format!("Invalid homeserver pubkey: {}", e) })?;
    
    let signer = sdk.signer(keypair);
    let signup_token = options.and_then(|o| o.signup_token);
    
    let session = signer.signup(&homeserver, signup_token.as_deref()).await?;
    
    let info = session.info();
    let session_info = PubkySessionInfo {
        pubkey: info.public_key().to_string(),
        capabilities: info.capabilities().iter().map(|c| c.to_string()).collect(),
        created_at: info.created_at(),
    };
    
    // Store session for later use
    let mut sessions = get_sessions().write().await;
    sessions.insert(session_info.pubkey.clone(), session);
    
    Ok(session_info)
}

/// Sign out and remove session
#[uniffi::export]
pub async fn pubky_signout(pubkey: String) -> Result<(), PubkyError> {
    let mut sessions = get_sessions().write().await;
    
    if let Some(session) = sessions.remove(&pubkey) {
        session.signout().await.map_err(|(e, _)| PubkyError::from(e))?;
    }
    
    Ok(())
}

/// Import a session from Pubky Ring
/// This is used when receiving a session from Pubky Ring via callback
/// 
/// - pubkey: The z-base32 encoded public key (for verification)
/// - session_secret: The full session token in format `<pubkey>:<cookie>` from Pubky Ring
#[uniffi::export]
pub fn pubky_import_session(_pubkey: String, session_secret: String) -> Result<PubkySessionInfo, PubkyError> {
    let sdk = get_sdk()?;
    
    // session_secret from Pubky Ring is already in the format `<pubkey>:<cookie>`
    // So we use it directly without modification
    let session_token = session_secret;
    
    // Use the global runtime to execute async operations
    // This is needed because PubkySession::import_secret internally uses Tokio features
    let runtime = crate::ensure_runtime();
    
    let client = sdk.client().clone();
    let result = runtime.block_on(async move {
        // Import the session using the SDK's import_secret method
        let session = PubkySession::import_secret(&session_token, Some(client)).await
            .map_err(|e| PubkyError::from(e))?;
        
        let info = session.info();
        let session_info = PubkySessionInfo {
            pubkey: info.public_key().to_string(),
            capabilities: info.capabilities().iter().map(|c| c.to_string()).collect(),
            created_at: info.created_at(),
        };
        
        // Store session for later use
        let mut sessions = get_sessions().write().await;
        sessions.insert(session_info.pubkey.clone(), session);
        
        Ok::<_, PubkyError>(session_info)
    })?;
    
    Ok(result)
}

/// Check if a session exists for a pubkey
#[uniffi::export]
pub async fn pubky_has_session(pubkey: String) -> bool {
    let sessions = get_sessions().read().await;
    sessions.contains_key(&pubkey)
}

/// Get session info for an active session
#[uniffi::export]
pub async fn pubky_get_session(pubkey: String) -> Result<Option<PubkySessionInfo>, PubkyError> {
    let sessions = get_sessions().read().await;
    
    if let Some(session) = sessions.get(&pubkey) {
        let info = session.info();
        Ok(Some(PubkySessionInfo {
            pubkey: info.public_key().to_string(),
            capabilities: info.capabilities().iter().map(|c| c.to_string()).collect(),
            created_at: info.created_at(),
        }))
    } else {
        Ok(None)
    }
}

/// List all active session pubkeys
#[uniffi::export]
pub async fn pubky_list_sessions() -> Vec<String> {
    let sessions = get_sessions().read().await;
    sessions.keys().cloned().collect()
}

/// Get data from authenticated storage
#[uniffi::export]
pub fn pubky_session_get(pubkey: String, path: String) -> Result<Vec<u8>, PubkyError> {
    let runtime = crate::ensure_runtime();
    
    runtime.block_on(async move {
        let sessions = get_sessions().read().await;
        
        let session = sessions.get(&pubkey).ok_or_else(|| PubkyError::Session {
            message: format!("No session found for pubkey: {}", pubkey),
        })?;
        
        let storage = session.storage();
        let response = storage.get(path).await?;
        let bytes = response.bytes().await
            .map_err(|e| PubkyError::Network { message: e.to_string() })?;
        
        Ok(bytes.to_vec())
    })
}

/// Put data to authenticated storage
#[uniffi::export]
pub fn pubky_session_put(pubkey: String, path: String, content: Vec<u8>) -> Result<(), PubkyError> {
    let runtime = crate::ensure_runtime();
    
    runtime.block_on(async move {
        let sessions = get_sessions().read().await;
        
        let session = sessions.get(&pubkey).ok_or_else(|| PubkyError::Session {
            message: format!("No session found for pubkey: {}", pubkey),
        })?;
        
        let storage = session.storage();
        storage.put(path, content).await?;
        
        Ok(())
    })
}

/// Delete data from authenticated storage
#[uniffi::export]
pub fn pubky_session_delete(pubkey: String, path: String) -> Result<(), PubkyError> {
    let runtime = crate::ensure_runtime();
    
    runtime.block_on(async move {
        let sessions = get_sessions().read().await;
        
        let session = sessions.get(&pubkey).ok_or_else(|| PubkyError::Session {
            message: format!("No session found for pubkey: {}", pubkey),
        })?;
        
        let storage = session.storage();
        storage.delete(path).await?;
        
        Ok(())
    })
}

/// List items in authenticated storage
#[uniffi::export]
pub fn pubky_session_list(pubkey: String, path: String) -> Result<Vec<PubkyListItem>, PubkyError> {
    let runtime = crate::ensure_runtime();
    
    runtime.block_on(async move {
        let sessions = get_sessions().read().await;
        
        let session = sessions.get(&pubkey).ok_or_else(|| PubkyError::Session {
            message: format!("No session found for pubkey: {}", pubkey),
        })?;
        
        let storage = session.storage();
        let builder = storage.list(path)?;
        let resources = builder.send().await?;
        
        let items = resources.into_iter().map(|r| {
            let path_str = r.path.as_str();
            let is_directory = path_str.ends_with('/');
            let name = path_str.trim_end_matches('/').split('/').next_back().unwrap_or(path_str).to_string();
            
            PubkyListItem {
                name,
                path: path_str.to_string(),
                is_directory,
            }
        }).collect();
        
        Ok(items)
    })
}

/// Get data from public storage (no authentication needed)
#[uniffi::export]
pub async fn pubky_public_get(uri: String) -> Result<Vec<u8>, PubkyError> {
    let sdk = get_sdk()?;
    let public = sdk.public_storage();
    
    let response = public.get(&uri).await?;
    let bytes = response.bytes().await
        .map_err(|e| PubkyError::Network { message: e.to_string() })?;
    
    Ok(bytes.to_vec())
}

/// List items in public storage
#[uniffi::export]
pub async fn pubky_public_list(uri: String) -> Result<Vec<PubkyListItem>, PubkyError> {
    let sdk = get_sdk()?;
    let public = sdk.public_storage();
    
    let builder = public.list(&uri)?;
    let resources = builder.send().await?;
    
    let items = resources.into_iter().map(|r| {
        let path_str = r.path.as_str();
        let is_directory = path_str.ends_with('/');
        let name = path_str.trim_end_matches('/').split('/').next_back().unwrap_or(path_str).to_string();
        
        PubkyListItem {
            name,
            path: path_str.to_string(),
            is_directory,
        }
    }).collect();
    
    Ok(items)
}

/// Fetch a profile from pubky.app profile.json
#[uniffi::export]
pub async fn pubky_fetch_profile(pubkey: String) -> Result<PubkyProfile, PubkyError> {
    let uri = format!("pubky://{}/pub/pubky.app/profile.json", pubkey);
    let data = pubky_public_get(uri).await?;
    
    let json: serde_json::Value = serde_json::from_slice(&data)
        .map_err(|e| PubkyError::InvalidInput { message: format!("Invalid profile JSON: {}", e) })?;
    
    Ok(PubkyProfile {
        name: json.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        bio: json.get("bio").and_then(|v| v.as_str()).map(|s| s.to_string()),
        image: json.get("image").and_then(|v| v.as_str()).map(|s| s.to_string()),
        links: json.get("links")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        status: json.get("status").and_then(|v| v.as_str()).map(|s| s.to_string()),
    })
}

/// Fetch follows list from pubky.app
#[uniffi::export]
pub async fn pubky_fetch_follows(pubkey: String) -> Result<Vec<String>, PubkyError> {
    let uri = format!("pubky://{}/pub/pubky.app/follows/", pubkey);
    let items = pubky_public_list(uri).await?;
    
    // Each follow is stored as a file named after the followed pubkey
    let follows = items.into_iter()
        .filter(|i| !i.is_directory)
        .map(|i| i.name)
        .collect();
    
    Ok(follows)
}

/// Generate a new random keypair
#[uniffi::export]
pub fn pubky_generate_keypair() -> PubkyKeypair {
    let keypair = Keypair::random();
    PubkyKeypair {
        secret_key_hex: hex::encode(keypair.secret_key()),
        public_key: keypair.public_key().to_string(),
    }
}

/// Get public key from secret key
#[uniffi::export]
pub fn pubky_public_key_from_secret(secret_key_hex: String) -> Result<String, PubkyError> {
    let secret_bytes = hex::decode(&secret_key_hex)
        .map_err(|e| PubkyError::InvalidInput { message: format!("Invalid hex: {}", e) })?;
    
    if secret_bytes.len() != 32 {
        return Err(PubkyError::InvalidInput {
            message: "Secret key must be 32 bytes".to_string(),
        });
    }
    
    let mut secret_arr = [0u8; 32];
    secret_arr.copy_from_slice(&secret_bytes);
    
    let keypair = Keypair::from_secret_key(&secret_arr);
    Ok(keypair.public_key().to_string())
}

/// Resolve a pubky to its homeserver
#[uniffi::export]
pub async fn pubky_resolve_homeserver(pubkey: String) -> Result<Option<String>, PubkyError> {
    let sdk = get_sdk()?;
    let pkdns = sdk.pkdns();
    
    let pk = PublicKey::try_from(pubkey.as_str())
        .map_err(|e| PubkyError::InvalidInput { message: format!("Invalid pubkey: {}", e) })?;
    
    let homeserver = pkdns.get_homeserver_of(&pk).await;
    Ok(homeserver.map(|h| h.to_string()))
}
