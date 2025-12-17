#[derive(uniffi::Record, Clone, Debug)]
pub struct PaykitSupportedMethod {
    pub method_id: String,
    pub endpoint: String,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct PaykitSupportedMethods {
    pub methods: Vec<PaykitSupportedMethod>,
}

impl From<paykit_lib::SupportedPayments> for PaykitSupportedMethods {
    fn from(sp: paykit_lib::SupportedPayments) -> Self {
        let methods = sp
            .to_list()
            .into_iter()
            .map(|(id, data)| PaykitSupportedMethod {
                method_id: id,
                endpoint: data,
            })
            .collect();
        Self { methods }
    }
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct PaykitReceiptFfi {
    pub receipt_id: String,
    pub payer: String,
    pub payee: String,
    pub method_id: String,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub created_at: i64,
    pub metadata_json: String,
}

impl TryFrom<paykit_interactive::PaykitReceipt> for PaykitReceiptFfi {
    type Error = serde_json::Error;

    fn try_from(r: paykit_interactive::PaykitReceipt) -> Result<Self, Self::Error> {
        Ok(Self {
            receipt_id: r.receipt_id,
            payer: r.payer.to_string(), // Debug format for now, or use inner string if not pubky
            payee: r.payee.to_string(),
            method_id: r.method_id.0,
            amount: r.amount,
            currency: r.currency,
            created_at: r.created_at,
            metadata_json: serde_json::to_string(&r.metadata)?,
        })
    }
}

impl TryFrom<PaykitReceiptFfi> for paykit_interactive::PaykitReceipt {
    type Error = String;

    fn try_from(r: PaykitReceiptFfi) -> Result<Self, Self::Error> {
        // We need a way to parse PublicKey.
        // Assuming paykit-lib PublicKey has FromStr or similar
        // For this implementation, we'll use a helper in interactive.rs or just parse here if possible.
        // PublicKey doesn't implement FromStr in the generic trait sense visible here easily without imports.
        // But wait, paykit_lib::PublicKey is re-exported.

        use std::str::FromStr;
        let payer = paykit_lib::PublicKey::from_str(&r.payer)
            .map_err(|e| format!("Invalid payer key: {}", e))?;
        let payee = paykit_lib::PublicKey::from_str(&r.payee)
            .map_err(|e| format!("Invalid payee key: {}", e))?;

        Ok(Self {
            receipt_id: r.receipt_id,
            payer,
            payee,
            method_id: paykit_lib::MethodId(r.method_id),
            amount: r.amount,
            currency: r.currency,
            created_at: r.created_at,
            metadata: serde_json::from_str(&r.metadata_json)
                .map_err(|e| format!("Invalid metadata JSON: {}", e))?,
        })
    }
}

/// Result of the smart checkout flow
#[derive(uniffi::Record, Clone, Debug)]
pub struct PaykitCheckoutResult {
    /// The payment method ID (e.g., "onchain", "lightning")
    pub method_id: String,
    /// The endpoint data (e.g., Bitcoin address or Lightning invoice)
    pub endpoint: String,
    /// Whether this is a private channel (true) or public directory (false)
    pub is_private: bool,
    /// Whether this requires interactive protocol (receipt negotiation)
    pub requires_interactive: bool,
}
