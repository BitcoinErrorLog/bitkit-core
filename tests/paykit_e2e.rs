//! E2E Tests for Paykit
//!
//! This test file runs the E2E test suite with mock homeserver.

mod e2e;

// Re-export tests so they run with cargo test
pub use e2e::*;

