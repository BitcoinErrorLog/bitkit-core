use crate::modules::paykit::errors::PaykitError;
use async_trait::async_trait;
use paykit_interactive::{PaykitReceipt, PaykitStorage, Result as InteractiveResult};
use paykit_lib::{MethodId, PublicKey};
use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

/// SQLite-backed implementation of PaykitStorage for bitkit-core.
///
/// Stores receipts and private endpoints in the activity database.
pub struct BitkitPaykitStorage {
    conn: Arc<Mutex<Connection>>,
}

impl BitkitPaykitStorage {
    pub fn new(db_path: &str) -> Result<Self, PaykitError> {
        let conn = Connection::open(db_path)
            .map_err(|e| PaykitError::Generic(format!("Failed to open database: {}", e)))?;

        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS paykit_receipts (
                receipt_id TEXT PRIMARY KEY,
                payer TEXT NOT NULL,
                payee TEXT NOT NULL,
                method_id TEXT NOT NULL,
                amount TEXT,
                currency TEXT,
                created_at INTEGER NOT NULL,
                metadata TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| PaykitError::Generic(format!("Failed to create receipts table: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS paykit_private_endpoints (
                peer TEXT NOT NULL,
                method_id TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (peer, method_id)
            )",
            [],
        )
        .map_err(|e| {
            PaykitError::Generic(format!("Failed to create private endpoints table: {}", e))
        })?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl PaykitStorage for BitkitPaykitStorage {
    async fn save_receipt(&self, receipt: &PaykitReceipt) -> InteractiveResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Mutex poisoned: {}", e))
        })?;

        let payer_str = format!("{:?}", receipt.payer); // PublicKey debug format
        let payee_str = format!("{:?}", receipt.payee);
        let metadata_str = serde_json::to_string(&receipt.metadata)
            .map_err(|e| paykit_interactive::InteractiveError::Serialization(e.to_string()))?;

        conn.execute(
            "INSERT OR REPLACE INTO paykit_receipts 
             (receipt_id, payer, payee, method_id, amount, currency, created_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &receipt.receipt_id,
                &payer_str,
                &payee_str,
                &receipt.method_id.0,
                &receipt.amount,
                &receipt.currency,
                receipt.created_at,
                &metadata_str,
            ),
        )
        .map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!(
                "Failed to save receipt: {}",
                e
            ))
        })?;

        Ok(())
    }

    async fn get_receipt(&self, receipt_id: &str) -> InteractiveResult<Option<PaykitReceipt>> {
        let conn = self.conn.lock().map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Mutex poisoned: {}", e))
        })?;

        let result: Option<(
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            String,
        )> = conn
            .query_row(
                "SELECT receipt_id, payer, payee, method_id, amount, currency, created_at, metadata
                 FROM paykit_receipts WHERE receipt_id = ?1",
                [receipt_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| {
                paykit_interactive::InteractiveError::Transport(format!(
                    "Failed to query receipt: {}",
                    e
                ))
            })?;

        if let Some((
            receipt_id,
            payer_str,
            payee_str,
            method_id_str,
            amount,
            currency,
            created_at,
            metadata_str,
        )) = result
        {
            // Parse PublicKey from string (this is a simplification - proper deserialization needed)
            // For now, we'll use a placeholder implementation
            let payer = parse_public_key(&payer_str).map_err(|e| {
                paykit_interactive::InteractiveError::Transport(format!("Invalid payer key: {}", e))
            })?;
            let payee = parse_public_key(&payee_str).map_err(|e| {
                paykit_interactive::InteractiveError::Transport(format!("Invalid payee key: {}", e))
            })?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)
                .map_err(|e| paykit_interactive::InteractiveError::Serialization(e.to_string()))?;

            Ok(Some(PaykitReceipt {
                receipt_id,
                payer,
                payee,
                method_id: MethodId(method_id_str),
                amount,
                currency,
                created_at,
                metadata,
            }))
        } else {
            Ok(None)
        }
    }

    async fn save_private_endpoint(
        &self,
        peer: &PublicKey,
        method: &MethodId,
        endpoint: &str,
    ) -> InteractiveResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Mutex poisoned: {}", e))
        })?;

        let peer_str = format!("{:?}", peer);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT OR REPLACE INTO paykit_private_endpoints 
             (peer, method_id, endpoint, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            (&peer_str, &method.0, endpoint, now),
        )
        .map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!(
                "Failed to save private endpoint: {}",
                e
            ))
        })?;

        Ok(())
    }

    async fn get_private_endpoint(
        &self,
        peer: &PublicKey,
        method: &MethodId,
    ) -> InteractiveResult<Option<String>> {
        let conn = self.conn.lock().map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Mutex poisoned: {}", e))
        })?;

        let peer_str = format!("{:?}", peer);

        let result: Option<String> = conn
            .query_row(
                "SELECT endpoint FROM paykit_private_endpoints 
                 WHERE peer = ?1 AND method_id = ?2",
                (&peer_str, &method.0),
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| {
                paykit_interactive::InteractiveError::Transport(format!(
                    "Failed to query private endpoint: {}",
                    e
                ))
            })?;

        Ok(result)
    }
    
    async fn list_receipts(&self) -> InteractiveResult<Vec<PaykitReceipt>> {
        let conn = self.conn.lock().map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Mutex poisoned: {}", e))
        })?;

        let mut stmt = conn.prepare(
            "SELECT receipt_id, payer, payee, method_id, amount, currency, created_at, metadata
             FROM paykit_receipts ORDER BY created_at DESC"
        ).map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Failed to prepare statement: {}", e))
        })?;

        let receipts = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
            ))
        }).map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Failed to query receipts: {}", e))
        })?;

        let mut result = Vec::new();
        for receipt_row in receipts {
            let (receipt_id, payer_str, payee_str, method_id_str, amount, currency, created_at, metadata_str) =
                receipt_row.map_err(|e| {
                    paykit_interactive::InteractiveError::Transport(format!("Failed to read row: {}", e))
                })?;

            let payer = parse_public_key(&payer_str).map_err(|e| {
                paykit_interactive::InteractiveError::Transport(format!("Invalid payer key: {}", e))
            })?;
            let payee = parse_public_key(&payee_str).map_err(|e| {
                paykit_interactive::InteractiveError::Transport(format!("Invalid payee key: {}", e))
            })?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)
                .map_err(|e| paykit_interactive::InteractiveError::Serialization(e.to_string()))?;

            result.push(PaykitReceipt {
                receipt_id,
                payer,
                payee,
                method_id: MethodId(method_id_str),
                amount,
                currency,
                created_at,
                metadata,
            });
        }

        Ok(result)
    }

    async fn list_private_endpoints_for_peer(
        &self,
        peer: &PublicKey,
    ) -> InteractiveResult<Vec<(MethodId, String)>> {
        let conn = self.conn.lock().map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Mutex poisoned: {}", e))
        })?;

        let peer_str = format!("{:?}", peer);

        let mut stmt = conn.prepare(
            "SELECT method_id, endpoint FROM paykit_private_endpoints WHERE peer = ?1"
        ).map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Failed to prepare statement: {}", e))
        })?;

        let endpoints = stmt.query_map([&peer_str], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Failed to query endpoints: {}", e))
        })?;

        let mut result = Vec::new();
        for endpoint_row in endpoints {
            let (method_id_str, endpoint) = endpoint_row.map_err(|e| {
                paykit_interactive::InteractiveError::Transport(format!("Failed to read row: {}", e))
            })?;
            result.push((MethodId(method_id_str), endpoint));
        }

        Ok(result)
    }

    async fn remove_private_endpoint(
        &self,
        peer: &PublicKey,
        method: &MethodId,
    ) -> InteractiveResult<()> {
        let conn = self.conn.lock().map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!("Mutex poisoned: {}", e))
        })?;

        let peer_str = format!("{:?}", peer);

        conn.execute(
            "DELETE FROM paykit_private_endpoints WHERE peer = ?1 AND method_id = ?2",
            (&peer_str, &method.0),
        ).map_err(|e| {
            paykit_interactive::InteractiveError::Transport(format!(
                "Failed to remove private endpoint: {}",
                e
            ))
        })?;

        Ok(())
    }
}

// Helper function for PublicKey parsing
fn parse_public_key(s: &str) -> Result<PublicKey, String> {
    use std::str::FromStr;
    parse_public_key(s).map_err(|e| format!("{}", e))
}
