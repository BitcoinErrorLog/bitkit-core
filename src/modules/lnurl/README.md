# LNURL Module

This module handles LNURL-related functionality, including Lightning Address invoice generation, LNURL-channel, LNURL-withdraw, and LNURL-auth protocols.

## Available Methods

- **`getLnurlInvoice`**: Generate an invoice from a Lightning Address
- **`createChannelRequestUrl`**: Create callback URL for LNURL-channel requests  
- **`createWithdrawCallbackUrl`**: Create callback URL for LNURL-withdraw requests
- **`lnurlAuth`**: Perform LNURL authentication with BIP32 key derivation

## Usage Examples

### Lightning Address Invoice Generation

#### iOS (Swift) Example
```swift
import BitkitCore

func generateInvoice() async {
    do {
        let invoice = try await getLnurlInvoice(address: "user@domain.com", amountSatoshis: 1000)
        print("Generated invoice: \(invoice)")
    } catch let error as LnurlError {
        switch error {
        case .invalidAddress:
            print("Invalid Lightning Address format")
        case .clientCreationFailed:
            print("Failed to create LNURL client")
        case .requestFailed:
            print("LNURL request failed")
        case .invalidResponse:
            print("Received invalid response from LNURL service")
        case .invalidAmount(let amount, let min, let max):
            print("Amount \(amount) is outside allowed range (\(min) - \(max) sats)")
        case .invoiceCreationFailed(let message):
            print("Failed to generate invoice: \(message)")
        }
    }
}
```

### LNURL-Channel Request

#### iOS (Swift) Example
```swift
import BitkitCore

func createChannelRequest() {
    do {
        let url = try createChannelRequestUrl(
            k1: "k1_value_from_lnurl_params",
            callback: "https://service.com/lnurl/channel/callback",
            localNodeId: "03abc123...", 
            isPrivate: false,
            cancel: false
        )
        print("Channel request URL: \(url)")
    } catch let error as LnurlError {
        print("Error creating channel request: \(error)")
    }
}
```

### LNURL-Withdraw Callback

#### iOS (Swift) Example  
```swift
import BitkitCore

func createWithdrawCallback() {
    do {
        let url = try createWithdrawCallbackUrl(
            k1: "k1_value_from_lnurl_params",
            callback: "https://service.com/lnurl/withdraw/callback", 
            paymentRequest: "lnbc1000n1p..."
        )
        print("Withdraw callback URL: \(url)")
    } catch let error as LnurlError {
        print("Error creating withdraw callback: \(error)")
    }
}
```

### LNURL-Auth Authentication

#### iOS (Swift) Example
```swift
import BitkitCore

func performLnurlAuth() async {
    do {
        let response = try await lnurlAuth(
            domain: "service.com",
            k1: "k1_challenge_from_service", 
            callback: "https://service.com/lnurl/auth/callback",
            bip32Mnemonic: "abandon abandon abandon...",
            network: Network.bitcoin,
            bip39Passphrase: nil
        )
        print("Auth response: \(response)")
    } catch let error as LnurlError {
        print("Authentication failed: \(error)")
    }
}
```

### Android (Kotlin) Examples

#### Lightning Address Invoice Generation
```kotlin
import com.synonym.bitkitcore.*

suspend fun generateInvoice() {
    try {
        val invoice = getLnurlInvoice("user@domain.com", 1000)
        println("Generated invoice: $invoice")
    } catch (e: LnurlError) {
        when (e) {
            is LnurlError.InvalidAddress -> println("Invalid Lightning Address format")
            is LnurlError.ClientCreationFailed -> println("Failed to create LNURL client")
            is LnurlError.RequestFailed -> println("LNURL request failed")
            is LnurlError.InvalidResponse -> println("Received invalid response from LNURL service")
            is LnurlError.InvalidAmount -> println(
                "Amount ${e.amountSatoshis} is outside allowed range " +
                "(${e.min} - ${e.max} sats)"
            )
            is LnurlError.InvoiceCreationFailed -> println("Failed to generate invoice: ${e.message}")
        }
    }
}
```

#### LNURL-Channel Request  
```kotlin
import com.synonym.bitkitcore.*

fun createChannelRequest() {
    try {
        val url = createChannelRequestUrl(
            k1 = "k1_value_from_lnurl_params",
            callback = "https://service.com/lnurl/channel/callback",
            localNodeId = "03abc123...",
            isPrivate = false,
            cancel = false
        )
        println("Channel request URL: $url")
    } catch (e: LnurlError) {
        println("Error creating channel request: $e")
    }
}
```

#### LNURL-Withdraw Callback
```kotlin
import com.synonym.bitkitcore.*

fun createWithdrawCallback() {
    try {
        val url = createWithdrawCallbackUrl(
            k1 = "k1_value_from_lnurl_params", 
            callback = "https://service.com/lnurl/withdraw/callback",
            paymentRequest = "lnbc1000n1p..."
        )
        println("Withdraw callback URL: $url")
    } catch (e: LnurlError) {
        println("Error creating withdraw callback: $e")
    }
}
```

#### LNURL-Auth Authentication
```kotlin
import com.synonym.bitkitcore.*

suspend fun performLnurlAuth() {
    try {
        val response = lnurlAuth(
            domain = "service.com",
            k1 = "k1_challenge_from_service",
            callback = "https://service.com/lnurl/auth/callback", 
            bip32Mnemonic = "abandon abandon abandon...",
            network = Network.BITCOIN,
            bip39Passphrase = null
        )
        println("Auth response: $response")
    } catch (e: LnurlError) {
        println("Authentication failed: $e")
    }
}
```

### Python Examples

#### Lightning Address Invoice Generation
```python
from bitkitcore import get_lnurl_invoice, LnurlError

async def generate_invoice():
    try:
        invoice = await get_lnurl_invoice("user@domain.com", 1000)  # 1000 sats
        print(f"Generated invoice: {invoice}")
    except LnurlError as e:
        if isinstance(e, LnurlError.InvalidAddress):
            print("Invalid Lightning Address format")
        elif isinstance(e, LnurlError.ClientCreationFailed):
            print("Failed to create LNURL client")
        elif isinstance(e, LnurlError.RequestFailed):
            print("LNURL request failed")
        elif isinstance(e, LnurlError.InvalidResponse):
            print("Received invalid response from LNURL service")
        elif isinstance(e, LnurlError.InvalidAmount):
            print(f"Amount {e.amount_satoshis} is outside allowed range " +
                  f"({e.min} - {e.max} sats)")
        elif isinstance(e, LnurlError.InvoiceCreationFailed):
            print(f"Failed to generate invoice: {e.message}")
```

#### LNURL-Channel Request
```python
from bitkitcore import create_channel_request_url, LnurlError

def create_channel_request():
    try:
        url = create_channel_request_url(
            k1="k1_value_from_lnurl_params",
            callback="https://service.com/lnurl/channel/callback",
            local_node_id="03abc123...",
            is_private=False,
            cancel=False
        )
        print(f"Channel request URL: {url}")
    except LnurlError as e:
        print(f"Error creating channel request: {e}")
```

#### LNURL-Withdraw Callback
```python
from bitkitcore import create_withdraw_callback_url, LnurlError

def create_withdraw_callback():
    try:
        url = create_withdraw_callback_url(
            k1="k1_value_from_lnurl_params",
            callback="https://service.com/lnurl/withdraw/callback",
            payment_request="lnbc1000n1p..."
        )
        print(f"Withdraw callback URL: {url}")
    except LnurlError as e:
        print(f"Error creating withdraw callback: {e}")
```

#### LNURL-Auth Authentication
```python
from bitkitcore import lnurl_auth, Network, LnurlError

async def perform_lnurl_auth():
    try:
        response = await lnurl_auth(
            domain="service.com",
            k1="k1_challenge_from_service",
            callback="https://service.com/lnurl/auth/callback",
            bip32_mnemonic="abandon abandon abandon...",
            network=Network.BITCOIN,
            bip39_passphrase=None
        )
        print(f"Auth response: {response}")
    except LnurlError as e:
        print(f"Authentication failed: {e}")
```

## Error Handling

### LnurlError
- `InvalidAddress`: The Lightning Address format is invalid
- `ClientCreationFailed`: Failed to create the LNURL client
- `RequestFailed`: The LNURL request failed
- `InvalidResponse`: Received an invalid response from LNURL service
- `InvalidAmount`: Amount is outside the allowed range, includes:
  - `amount_satoshis`: The invalid amount that was provided
  - `min`: Minimum allowed amount in satoshis
  - `max`: Maximum allowed amount in satoshis
- `InvoiceCreationFailed`: Failed to generate the invoice, includes:
  - `message`: Detailed error message explaining the failure