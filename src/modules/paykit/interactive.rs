use crate::modules::paykit::errors::PaykitError;
use crate::modules::paykit::receipt_generator::BitkitReceiptGenerator;
use crate::modules::paykit::storage::BitkitPaykitStorage;
use crate::modules::paykit::types::PaykitReceiptFfi;
use paykit_interactive::transport::PubkyNoiseChannel;
use paykit_interactive::{PaykitInteractiveManager, PaykitReceipt};
use pubky_noise::{DummyRing, NoiseClient};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream;

#[derive(uniffi::Object)]
pub struct PaykitInteractive {
    inner: PaykitInteractiveManager,
    secret_key: Vec<u8>,
}

#[uniffi::export]
impl PaykitInteractive {
    #[uniffi::constructor]
    pub fn new(db_path: String, secret_key_hex: String) -> Result<Self, PaykitError> {
        let storage = Arc::new(Box::new(BitkitPaykitStorage::new(&db_path)?)
            as Box<dyn paykit_interactive::PaykitStorage>);
        let generator = Arc::new(Box::new(BitkitReceiptGenerator::new())
            as Box<dyn paykit_interactive::ReceiptGenerator>);

        let secret_key = hex::decode(secret_key_hex)
            .map_err(|e| PaykitError::Generic(format!("Invalid secret: {}", e)))?;

        if secret_key.len() != 32 {
            return Err(PaykitError::Generic(
                "Secret key must be 32 bytes".to_string(),
            ));
        }

        Ok(Self {
            inner: PaykitInteractiveManager::new(storage, generator),
            secret_key,
        })
    }

    /// Initiate an interactive payment flow with a peer over TCP/Noise.
    ///
    /// * `host`: IP address or hostname of the peer.
    /// * `port`: Port number.
    /// * `peer_pubkey`: Pubky ID (public key) of the peer.
    /// * `receipt`: Provisional receipt details (amount, method, metadata).
    pub async fn initiate_payment(
        &self,
        host: String,
        port: u16,
        peer_pubkey: String,
        receipt: PaykitReceiptFfi,
    ) -> Result<PaykitReceiptFfi, PaykitError> {
        // 1. Connect TCP
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| PaykitError::Transport(format!("TCP connect failed: {}", e)))?;

        // 2. Setup Noise Client
        // Using DummyRing for now as we have the raw secret.
        let client_kid = "primary";
        let seed: [u8; 32] = self.secret_key.clone().try_into().unwrap();

        // Note: In a real app, device_id should probably be persistent or derived
        let ring = Arc::new(DummyRing::new_with_device(
            seed,
            client_kid.to_string(),
            vec![0; 32], // device_id placeholder
            0,           // epoch
        ));

        let noise_client = NoiseClient::new_direct(client_kid, vec![0; 32], ring);

        // 3. Parse peer key
        let peer_pk = paykit_lib::PublicKey::from_str(&peer_pubkey)
            .map_err(|e| PaykitError::InvalidPublicKey(e.to_string()))?;

        let peer_bytes = peer_pk.as_bytes();

        // 4. Establish Noise Channel
        let mut channel = PubkyNoiseChannel::connect(&noise_client, stream, peer_bytes)
            .await
            .map_err(|e| PaykitError::Transport(e.to_string()))?;

        // 5. Convert Receipt
        let provisional: PaykitReceipt = receipt
            .try_into()
            .map_err(|e: String| PaykitError::Generic(e))?;

        // 6. Execute Flow
        let final_receipt = self
            .inner
            .initiate_payment(&mut channel, provisional)
            .await
            .map_err(|e| PaykitError::Transport(e.to_string()))?;

        // 7. Convert back
        PaykitReceiptFfi::try_from(final_receipt).map_err(|e| PaykitError::Generic(e.to_string()))
    }
}
