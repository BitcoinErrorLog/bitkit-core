//! E2E Test Harness
//!
//! Provides a complete test environment for end-to-end testing of Paykit flows.

use super::mock_homeserver::{MockHomeserver, MockPaymentEndpoint, MockProfile};
use pubky::Keypair;
use tempfile::TempDir;

/// Test session representing a user in the test environment
pub struct TestSession {
    pub keypair: Keypair,
    pub pubkey_z32: String,
    pub db_path: String,
    temp_dir: TempDir,
}

impl TestSession {
    /// Create a new test session with a random identity
    pub fn new() -> Self {
        Self::with_seed(None)
    }

    /// Create a test session with a deterministic identity from a seed
    pub fn with_seed(seed: Option<u8>) -> Self {
        let keypair = if let Some(s) = seed {
            let mut seed_bytes = [0u8; 32];
            seed_bytes[0] = s;
            // Use the seed to generate a deterministic keypair
            Keypair::from_secret_key(&seed_bytes)
        } else {
            Keypair::random()
        };

        let pubkey_z32 = keypair.public_key().to_z32();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

        Self {
            keypair,
            pubkey_z32,
            db_path,
            temp_dir,
        }
    }

    /// Get the secret key as hex
    pub fn secret_key_hex(&self) -> String {
        hex::encode(self.keypair.secret_key())
    }
}

impl Default for TestSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete E2E test harness with mock homeserver and test sessions
pub struct E2eTestHarness {
    pub mock_homeserver: MockHomeserver,
    pub bitkit_session: TestSession,
    pub pubky_ring_session: TestSession,
    cleanup_actions: Vec<Box<dyn FnOnce()>>,
}

impl E2eTestHarness {
    /// Create a new E2E test harness
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mock_homeserver = MockHomeserver::new().await?;
        let bitkit_session = TestSession::with_seed(Some(1));
        let pubky_ring_session = TestSession::with_seed(Some(2));

        Ok(Self {
            mock_homeserver,
            bitkit_session,
            pubky_ring_session,
            cleanup_actions: Vec::new(),
        })
    }

    /// Setup the Bitkit session's profile and endpoints in the mock homeserver
    pub fn setup_bitkit_profile(&self, name: &str) {
        self.mock_homeserver.add_profile(
            &self.bitkit_session.pubkey_z32,
            MockProfile {
                name: Some(name.to_string()),
                bio: Some("Bitkit test user".to_string()),
                image: None,
                links: vec![],
            },
        );
    }

    /// Setup payment endpoints for the Bitkit session
    pub fn setup_bitkit_endpoints(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.mock_homeserver.add_endpoint(
            &self.bitkit_session.pubkey_z32,
            MockPaymentEndpoint {
                method_id: "lightning".to_string(),
                endpoint: "lnbc1000n1test_bitkit".to_string(),
                created_at: now,
            },
        );

        self.mock_homeserver.add_endpoint(
            &self.bitkit_session.pubkey_z32,
            MockPaymentEndpoint {
                method_id: "onchain".to_string(),
                endpoint: "bc1qtest_bitkit".to_string(),
                created_at: now,
            },
        );
    }

    /// Setup a peer profile with payment endpoints
    pub fn setup_peer(&self, name: &str) -> TestSession {
        let peer = TestSession::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.mock_homeserver.add_profile(
            &peer.pubkey_z32,
            MockProfile {
                name: Some(name.to_string()),
                bio: Some(format!("{} test peer", name)),
                image: None,
                links: vec![],
            },
        );

        self.mock_homeserver.add_endpoint(
            &peer.pubkey_z32,
            MockPaymentEndpoint {
                method_id: "lightning".to_string(),
                endpoint: format!("lnbc1000n1_{}", name.to_lowercase()),
                created_at: now,
            },
        );

        peer
    }

    /// Setup follow relationships
    pub fn setup_follow(&self, follower: &str, following: &str) {
        self.mock_homeserver.add_follow(follower, following);
    }

    /// Get the database path for manual initialization
    /// Note: init_db uses block_on internally, so call it from a sync context
    pub fn get_db_path(&self) -> &str {
        &self.bitkit_session.db_path
    }

    /// Add a cleanup action to run when the harness is dropped
    pub fn add_cleanup<F: FnOnce() + 'static>(&mut self, action: F) {
        self.cleanup_actions.push(Box::new(action));
    }

    /// Run cleanup and shutdown
    pub fn cleanup(mut self) {
        // Run cleanup actions in reverse order
        while let Some(action) = self.cleanup_actions.pop() {
            action();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_harness_creation() {
        let harness = E2eTestHarness::new().await.unwrap();
        assert!(!harness.bitkit_session.pubkey_z32.is_empty());
        assert!(!harness.pubky_ring_session.pubkey_z32.is_empty());
        assert_ne!(harness.bitkit_session.pubkey_z32, harness.pubky_ring_session.pubkey_z32);
    }

    #[tokio::test]
    async fn test_setup_bitkit_profile() {
        let harness = E2eTestHarness::new().await.unwrap();
        harness.setup_bitkit_profile("Test Bitkit User");

        let profile = harness.mock_homeserver.get_profile(&harness.bitkit_session.pubkey_z32);
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().name, Some("Test Bitkit User".to_string()));
    }

    #[tokio::test]
    async fn test_setup_endpoints() {
        let harness = E2eTestHarness::new().await.unwrap();
        harness.setup_bitkit_endpoints();

        let endpoints = harness.mock_homeserver.get_endpoints(&harness.bitkit_session.pubkey_z32);
        assert_eq!(endpoints.len(), 2);
        assert!(endpoints.iter().any(|e| e.method_id == "lightning"));
        assert!(endpoints.iter().any(|e| e.method_id == "onchain"));
    }

    #[tokio::test]
    async fn test_setup_peer() {
        let harness = E2eTestHarness::new().await.unwrap();
        let peer = harness.setup_peer("Alice");

        let profile = harness.mock_homeserver.get_profile(&peer.pubkey_z32);
        assert!(profile.is_some());

        let endpoints = harness.mock_homeserver.get_endpoints(&peer.pubkey_z32);
        assert!(!endpoints.is_empty());
    }

    #[test]
    fn test_get_db_path() {
        // Use sync test since init_db uses block_on internally
        let rt = tokio::runtime::Runtime::new().unwrap();
        let harness = rt.block_on(E2eTestHarness::new()).unwrap();
        let db_path = harness.get_db_path();
        assert!(!db_path.is_empty());
        // Actually init in sync context
        bitkitcore::init_db(db_path.to_string()).unwrap();
    }

    #[test]
    fn test_session_deterministic() {
        let session1 = TestSession::with_seed(Some(42));
        let session2 = TestSession::with_seed(Some(42));
        assert_eq!(session1.pubkey_z32, session2.pubkey_z32);
    }

    #[test]
    fn test_session_different_seeds() {
        let session1 = TestSession::with_seed(Some(1));
        let session2 = TestSession::with_seed(Some(2));
        assert_ne!(session1.pubkey_z32, session2.pubkey_z32);
    }
}

