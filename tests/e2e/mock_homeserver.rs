//! Mock Homeserver for E2E Testing
//!
//! Provides a lightweight HTTP server that implements the Pubky homeserver API
//! for testing Paykit flows without network access.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// Profile data stored in the mock homeserver
#[derive(Debug, Clone, Default)]
pub struct MockProfile {
    pub name: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub links: Vec<String>,
}

/// Payment endpoint data
#[derive(Debug, Clone)]
pub struct MockPaymentEndpoint {
    pub method_id: String,
    pub endpoint: String,
    pub created_at: u64,
}

/// Session info for authenticated users
#[derive(Debug, Clone)]
pub struct MockSession {
    pub pubkey: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub capabilities: Vec<String>,
}

/// Mock homeserver state
pub struct MockHomeserver {
    pub port: u16,
    pub profiles: Arc<Mutex<HashMap<String, MockProfile>>>,
    pub endpoints: Arc<Mutex<HashMap<String, Vec<MockPaymentEndpoint>>>>,
    pub sessions: Arc<Mutex<HashMap<String, MockSession>>>,
    pub follows: Arc<Mutex<HashMap<String, Vec<String>>>>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl MockHomeserver {
    /// Create a new mock homeserver on a random available port
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_port(0).await
    }

    /// Create a new mock homeserver on a specific port
    pub async fn with_port(port: u16) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let profiles = Arc::new(Mutex::new(HashMap::new()));
        let endpoints = Arc::new(Mutex::new(HashMap::new()));
        let sessions = Arc::new(Mutex::new(HashMap::new()));
        let follows = Arc::new(Mutex::new(HashMap::new()));

        // For simplicity, we return a mock that doesn't actually start a server
        // In a full implementation, we'd use axum or warp here
        Ok(Self {
            port: if port == 0 { 8888 } else { port },
            profiles,
            endpoints,
            sessions,
            follows,
            shutdown_tx: None,
        })
    }

    /// Get the base URL for this mock homeserver
    pub fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Add a profile to the mock homeserver
    pub fn add_profile(&self, pubkey: &str, profile: MockProfile) {
        let mut profiles = self.profiles.lock().unwrap();
        profiles.insert(pubkey.to_string(), profile);
    }

    /// Get a profile from the mock homeserver
    pub fn get_profile(&self, pubkey: &str) -> Option<MockProfile> {
        let profiles = self.profiles.lock().unwrap();
        profiles.get(pubkey).cloned()
    }

    /// Add a payment endpoint for a pubkey
    pub fn add_endpoint(&self, pubkey: &str, endpoint: MockPaymentEndpoint) {
        let mut endpoints = self.endpoints.lock().unwrap();
        endpoints
            .entry(pubkey.to_string())
            .or_insert_with(Vec::new)
            .push(endpoint);
    }

    /// Get payment endpoints for a pubkey
    pub fn get_endpoints(&self, pubkey: &str) -> Vec<MockPaymentEndpoint> {
        let endpoints = self.endpoints.lock().unwrap();
        endpoints.get(pubkey).cloned().unwrap_or_default()
    }

    /// Add a follow relationship
    pub fn add_follow(&self, follower: &str, following: &str) {
        let mut follows = self.follows.lock().unwrap();
        follows
            .entry(follower.to_string())
            .or_insert_with(Vec::new)
            .push(following.to_string());
    }

    /// Get follows for a pubkey
    pub fn get_follows(&self, pubkey: &str) -> Vec<String> {
        let follows = self.follows.lock().unwrap();
        follows.get(pubkey).cloned().unwrap_or_default()
    }

    /// Create a session for a pubkey
    pub fn create_session(&self, pubkey: &str, capabilities: Vec<String>) -> MockSession {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = MockSession {
            pubkey: pubkey.to_string(),
            created_at: now,
            expires_at: now + 3600, // 1 hour
            capabilities,
        };

        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(pubkey.to_string(), session.clone());
        session
    }

    /// Check if a session is valid
    pub fn is_session_valid(&self, pubkey: &str) -> bool {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(pubkey) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            session.expires_at > now
        } else {
            false
        }
    }

    /// Clear all data from the mock homeserver
    pub fn clear_all(&self) {
        self.profiles.lock().unwrap().clear();
        self.endpoints.lock().unwrap().clear();
        self.sessions.lock().unwrap().clear();
        self.follows.lock().unwrap().clear();
    }

    /// Shutdown the mock homeserver
    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for MockHomeserver {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_homeserver_creation() {
        let server = MockHomeserver::new().await.unwrap();
        assert!(server.port > 0);
    }

    #[tokio::test]
    async fn test_add_and_get_profile() {
        let server = MockHomeserver::new().await.unwrap();
        let pubkey = "test_pubkey_123";

        server.add_profile(
            pubkey,
            MockProfile {
                name: Some("Test User".to_string()),
                bio: Some("A test user".to_string()),
                image: None,
                links: vec![],
            },
        );

        let profile = server.get_profile(pubkey);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().name, Some("Test User".to_string()));
    }

    #[tokio::test]
    async fn test_add_and_get_endpoints() {
        let server = MockHomeserver::new().await.unwrap();
        let pubkey = "test_pubkey_456";

        server.add_endpoint(
            pubkey,
            MockPaymentEndpoint {
                method_id: "lightning".to_string(),
                endpoint: "lnbc1000n1test".to_string(),
                created_at: 1700000000,
            },
        );

        let endpoints = server.get_endpoints(pubkey);
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].method_id, "lightning");
    }

    #[tokio::test]
    async fn test_follows() {
        let server = MockHomeserver::new().await.unwrap();
        let follower = "user_a";
        let following = "user_b";

        server.add_follow(follower, following);

        let follows = server.get_follows(follower);
        assert_eq!(follows.len(), 1);
        assert_eq!(follows[0], following);
    }

    #[tokio::test]
    async fn test_session_management() {
        let server = MockHomeserver::new().await.unwrap();
        let pubkey = "session_user";

        assert!(!server.is_session_valid(pubkey));

        server.create_session(pubkey, vec!["read".to_string(), "write".to_string()]);

        assert!(server.is_session_valid(pubkey));
    }

    #[tokio::test]
    async fn test_clear_all() {
        let server = MockHomeserver::new().await.unwrap();
        
        server.add_profile("user1", MockProfile::default());
        server.add_endpoint("user1", MockPaymentEndpoint {
            method_id: "lightning".to_string(),
            endpoint: "lnbc...".to_string(),
            created_at: 0,
        });

        server.clear_all();

        assert!(server.get_profile("user1").is_none());
        assert!(server.get_endpoints("user1").is_empty());
    }
}

