use crate::modules::paykit::errors::PaykitError;
use async_trait::async_trait;
use paykit_interactive::{PaykitReceipt, ReceiptGenerator, Result as InteractiveResult};

/// Bitkit implementation of ReceiptGenerator.
///
/// This implementation generates actual payment credentials (invoices, addresses)
/// when a receipt request is received.
pub struct BitkitReceiptGenerator {
    // TODO: Add references to wallet/blocktank services when integrating
    // blocktank_client: Arc<BlocktankClient>,
    // wallet: Arc<BitcoinWallet>,
}

impl BitkitReceiptGenerator {
    pub fn new() -> Self {
        Self {
            // Initialize with actual services when available
        }
    }

    /// Generate a Lightning invoice for the requested amount.
    ///
    /// This is a placeholder implementation. In production, this should:
    /// 1. Call Blocktank to create an invoice
    /// 2. Store the invoice in ActivityDB
    /// 3. Monitor for payment
    async fn generate_lightning_invoice(
        &self,
        amount_sats: u64,
        description: &str,
    ) -> Result<String, PaykitError> {
        // TODO: Replace with actual Blocktank call
        // let invoice = self.blocktank_client.create_invoice(amount_sats, description).await?;

        // Placeholder for development
        Ok(format!("lnbc{}1...placeholder", amount_sats))
    }

    /// Generate an on-chain address for the requested amount.
    ///
    /// This is a placeholder implementation. In production, this should:
    /// 1. Derive a new address from the wallet
    /// 2. Store the address in ActivityDB
    /// 3. Monitor for payment
    async fn generate_onchain_address(&self) -> Result<String, PaykitError> {
        // TODO: Replace with actual wallet call
        // let address = self.wallet.get_new_address().await?;

        // Placeholder for development
        Ok("bc1q...placeholder".to_string())
    }
}

#[async_trait]
impl ReceiptGenerator for BitkitReceiptGenerator {
    async fn generate_receipt(&self, request: &PaykitReceipt) -> InteractiveResult<PaykitReceipt> {
        // 1. Validate the request
        let amount = request
            .amount
            .as_ref()
            .and_then(|a| a.parse::<u64>().ok())
            .unwrap_or(0);

        if amount == 0 {
            return Err(paykit_interactive::InteractiveError::Protocol(
                "Amount must be specified and non-zero".into(),
            ));
        }

        // 2. Generate payment credential based on method
        let mut receipt = request.clone();
        let mut metadata = request.metadata.clone();

        match request.method_id.0.as_str() {
            "lightning" => {
                // Generate Lightning invoice
                let description = metadata
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Payment request");

                let invoice = self
                    .generate_lightning_invoice(amount, description)
                    .await
                    .map_err(|e| {
                        paykit_interactive::InteractiveError::Transport(format!(
                            "Failed to generate invoice: {}",
                            e
                        ))
                    })?;

                // Add invoice to metadata
                if let Some(obj) = metadata.as_object_mut() {
                    obj.insert("invoice".to_string(), serde_json::Value::String(invoice));
                    obj.insert(
                        "generated_at".to_string(),
                        serde_json::Value::Number(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                .into(),
                        ),
                    );
                }
            }
            "onchain" => {
                // Generate on-chain address
                let address = self.generate_onchain_address().await.map_err(|e| {
                    paykit_interactive::InteractiveError::Transport(format!(
                        "Failed to generate address: {}",
                        e
                    ))
                })?;

                // Add address to metadata
                if let Some(obj) = metadata.as_object_mut() {
                    obj.insert("address".to_string(), serde_json::Value::String(address));
                    obj.insert(
                        "generated_at".to_string(),
                        serde_json::Value::Number(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                                .into(),
                        ),
                    );
                }
            }
            method => {
                return Err(paykit_interactive::InteractiveError::Protocol(format!(
                    "Unsupported payment method: {}",
                    method
                )));
            }
        }

        receipt.metadata = metadata;

        // 3. Add server-side signature (future enhancement)
        // let signature = self.sign_receipt(&receipt).await?;
        // receipt.metadata["signature"] = signature.into();

        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paykit_lib::{MethodId, PublicKey};
    use serde_json::json;

    fn test_pubkey(s: &str) -> PublicKey {
        #[cfg(feature = "pubky")]
        {
            use std::str::FromStr;
            let valid_key = "8".repeat(52);
            PublicKey::from_str(&valid_key)
                .unwrap_or_else(|_| PublicKey::from_str(s).expect("Invalid test key"))
        }

        #[cfg(not(feature = "pubky"))]
        {
            PublicKey(s.to_string())
        }
    }

    #[tokio::test]
    async fn test_generate_lightning_receipt() {
        let generator = BitkitReceiptGenerator::new();

        let request = PaykitReceipt::new(
            "receipt_001".to_string(),
            test_pubkey("payer"),
            test_pubkey("payee"),
            MethodId("lightning".to_string()),
            Some("1000".to_string()),
            Some("SAT".to_string()),
            json!({"description": "Test payment"}),
        );

        let result = generator.generate_receipt(&request).await.unwrap();

        // Verify invoice was added
        assert!(result.metadata.get("invoice").is_some());
        assert!(result.metadata.get("generated_at").is_some());
    }

    #[tokio::test]
    async fn test_generate_onchain_receipt() {
        let generator = BitkitReceiptGenerator::new();

        let request = PaykitReceipt::new(
            "receipt_002".to_string(),
            test_pubkey("payer"),
            test_pubkey("payee"),
            MethodId("onchain".to_string()),
            Some("50000".to_string()),
            Some("SAT".to_string()),
            json!({}),
        );

        let result = generator.generate_receipt(&request).await.unwrap();

        // Verify address was added
        assert!(result.metadata.get("address").is_some());
        assert!(result.metadata.get("generated_at").is_some());
    }

    #[tokio::test]
    async fn test_unsupported_method_error() {
        let generator = BitkitReceiptGenerator::new();

        let request = PaykitReceipt::new(
            "receipt_003".to_string(),
            test_pubkey("payer"),
            test_pubkey("payee"),
            MethodId("unsupported".to_string()),
            Some("1000".to_string()),
            Some("SAT".to_string()),
            json!({}),
        );

        let result = generator.generate_receipt(&request).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Unsupported payment method"));
        }
    }

    #[tokio::test]
    async fn test_zero_amount_error() {
        let generator = BitkitReceiptGenerator::new();

        let request = PaykitReceipt::new(
            "receipt_004".to_string(),
            test_pubkey("payer"),
            test_pubkey("payee"),
            MethodId("lightning".to_string()),
            Some("0".to_string()),
            Some("SAT".to_string()),
            json!({}),
        );

        let result = generator.generate_receipt(&request).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Amount must be specified"));
        }
    }
}
