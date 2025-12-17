# Paykit Integration Guide

This document describes how to integrate Paykit functionality into iOS and Android apps using the bitkit-core FFI.

## Overview

Paykit enables peer-to-peer payments via the Pubky network. It provides:

- **Payment Discovery**: Find payment methods for any Pubky identity
- **Interactive Payments**: Request receipts via Noise channels for privacy
- **Auto-Pay Evaluation**: Set spending limits and automate recurring payments
- **Directory Operations**: Publish and discover payment endpoints

## Architecture

```
┌─────────────────────────────────────────────┐
│           Mobile App (iOS/Android)          │
│  ┌─────────────────────────────────────────┐│
│  │          PaykitService                  ││
│  │    (calls FFI functions)                ││
│  └────────────────┬────────────────────────┘│
└───────────────────┼─────────────────────────┘
                    │ FFI
┌───────────────────▼─────────────────────────┐
│              bitkit-core                    │
│  ┌─────────────────────────────────────────┐│
│  │       paykit module                     ││
│  │  - implementation.rs (FFI exports)      ││
│  │  - storage.rs (session/keys)            ││
│  │  - receipt_generator.rs (payments)      ││
│  └────────────────┬────────────────────────┘│
└───────────────────┼─────────────────────────┘
                    │
┌───────────────────▼─────────────────────────┐
│           paykit-rs Libraries               │
│  - paykit-lib (directory operations)        │
│  - paykit-interactive (Noise payments)      │
└───────────────────┬─────────────────────────┘
                    │
┌───────────────────▼─────────────────────────┐
│              Pubky Network                  │
│  - Homeserver (profile storage)             │
│  - DHT (public key resolution)              │
└─────────────────────────────────────────────┘
```

## Quick Start

### 1. Initialize Paykit

Before using Paykit, initialize with user credentials:

**iOS (Swift):**
```swift
func initializePaykit() async throws {
    let secretKeyHex = getSecretKeyFromKeychain()
    let homeserverPubkey = getHomeserverPubkey()
    
    try await paykit_initialize(
        secretKeyHex: secretKeyHex,
        homeserverPubkey: homeserverPubkey
    )
}
```

**Android (Kotlin):**
```kotlin
suspend fun initializePaykit() {
    val secretKeyHex = getSecretKeyFromKeychain()
    val homeserverPubkey = getHomeserverPubkey()
    
    paykitInitialize(
        secretKeyHex = secretKeyHex,
        homeserverPubkey = homeserverPubkey
    )
}
```

### 2. Publish Payment Endpoints

Set your public payment endpoints so others can pay you:

**iOS (Swift):**
```swift
func publishEndpoints() async throws {
    // Publish Lightning address
    try await paykit_set_endpoint(
        methodId: "lightning",
        endpoint: "lnbc1..."
    )
    
    // Publish Bitcoin address
    try await paykit_set_endpoint(
        methodId: "onchain",
        endpoint: "bc1q..."
    )
}
```

**Android (Kotlin):**
```kotlin
suspend fun publishEndpoints() {
    paykitSetEndpoint(
        methodId = "lightning",
        endpoint = "lnbc1..."
    )
    
    paykitSetEndpoint(
        methodId = "onchain",
        endpoint = "bc1q..."
    )
}
```

### 3. Discover Payment Methods

Find how to pay someone:

**iOS (Swift):**
```swift
func discoverPaymentMethods(forPubkey pubkey: String) async throws -> PaykitSupportedMethods {
    return try await paykit_get_supported_methods_for_key(pubkey: pubkey)
}
```

**Android (Kotlin):**
```kotlin
suspend fun discoverPaymentMethods(pubkey: String): PaykitSupportedMethods {
    return paykitGetSupportedMethodsForKey(pubkey = pubkey)
}
```

## Payment Discovery Flow

### Standard Flow

```
1. User enters recipient's Pubky ID
2. App calls paykit_get_supported_methods_for_key()
3. Returns list of available payment methods:
   - lightning: BOLT11 invoice or Lightning Address
   - onchain: Bitcoin address
   - paypal: PayPal email (future)
   - etc.
4. User selects preferred method
5. App calls paykit_get_endpoint_for_key_and_method()
6. Returns the payment endpoint
7. App proceeds to payment
```

### Smart Checkout Flow

For an optimized checkout experience:

**iOS (Swift):**
```swift
func smartCheckout(recipientPubkey: String, preferredMethod: String?) async throws -> PaykitCheckoutResult {
    return try await paykit_smart_checkout(
        pubkey: recipientPubkey,
        preferredMethod: preferredMethod
    )
}

// Usage
let result = try await smartCheckout(recipientPubkey: "o1gg96...", preferredMethod: "lightning")

if result.isPrivate {
    // Private offer found - more privacy
    let endpoint = result.endpoint
} else {
    // Public directory endpoint
    let endpoint = result.endpoint
}
```

Smart checkout:
1. Checks for private offers first (more privacy)
2. Falls back to public directory
3. Returns best available method

## Interactive Payment Flow

For payments requiring receipt generation:

### Sequence Diagram

```
┌────────┐                 ┌────────┐                 ┌────────┐
│  Payer │                 │ Payee  │                 │  DHT   │
└───┬────┘                 └───┬────┘                 └───┬────┘
    │                          │                          │
    │  1. Resolve Noise key    │                          │
    │─────────────────────────►│                          │
    │                          │                          │
    │  2. Establish channel    │                          │
    │◄────────────────────────►│                          │
    │                          │                          │
    │  3. Request receipt      │                          │
    │─────────────────────────►│                          │
    │                          │                          │
    │  4. Receipt + invoice    │                          │
    │◄─────────────────────────│                          │
    │                          │                          │
    │  5. Pay invoice          │                          │
    │─────────────────────────►│                          │
    │                          │                          │
    │  6. Confirmation         │                          │
    │◄─────────────────────────│                          │
    │                          │                          │
```

### Implementation

Interactive payments use the Noise protocol for encrypted communication:

**iOS (Swift):**
```swift
class InteractivePaymentService {
    func requestReceipt(
        fromPubkey: String,
        amount: UInt64,
        method: String
    ) async throws -> PaykitReceipt {
        // 1. Resolve peer's Noise public key
        let noiseKey = try await resolveNoiseKey(pubkey: fromPubkey)
        
        // 2. Establish encrypted channel
        let channel = try await establishNoiseChannel(peerKey: noiseKey)
        
        // 3. Send receipt request
        let request = PaykitReceiptRequest(
            amount: amount,
            method: method,
            metadata: ["description": "Payment request"]
        )
        
        // 4. Receive receipt with invoice
        return try await channel.requestReceipt(request)
    }
}
```

## Auto-Pay Evaluation

### Setting Spending Limits

Configure spending limits for automatic payments:

```swift
struct SpendingLimits {
    var globalDaily: UInt64 = 100_000    // 100k sats/day
    var globalMonthly: UInt64 = 1_000_000 // 1M sats/month
    var perPeer: [String: UInt64] = [:]   // Per-peer limits
}
```

### Evaluating Auto-Pay

Check if a payment can be auto-approved:

```swift
func canAutoPay(
    toPubkey: String,
    amount: UInt64
) async throws -> Bool {
    let limits = getSpendingLimits()
    let todaySpent = getTodaySpending()
    
    // Check global limit
    if todaySpent + amount > limits.globalDaily {
        return false
    }
    
    // Check peer-specific limit
    if let peerLimit = limits.perPeer[toPubkey] {
        let peerSpent = getPeerSpending(toPubkey)
        if peerSpent + amount > peerLimit {
            return false
        }
    }
    
    return true
}
```

## Subscription Management

### Creating a Subscription

```swift
struct Subscription {
    let id: String
    let recipientPubkey: String
    let amount: UInt64
    let interval: SubscriptionInterval // daily, weekly, monthly
    let methodId: String
    let autoPayEnabled: Bool
}

func createSubscription(_ subscription: Subscription) async throws {
    // Store subscription locally
    // Set up background task for recurring payments
}
```

### Processing Subscriptions

```swift
func processSubscription(_ subscription: Subscription) async throws {
    // 1. Check if due
    guard isSubscriptionDue(subscription) else { return }
    
    // 2. Discover endpoint
    let endpoint = try await paykit_get_endpoint_for_key_and_method(
        pubkey: subscription.recipientPubkey,
        methodId: subscription.methodId
    )
    
    // 3. Check auto-pay
    if subscription.autoPayEnabled && canAutoPay(subscription.recipientPubkey, subscription.amount) {
        // Auto-approve and pay
        try await executePayment(endpoint: endpoint, amount: subscription.amount)
    } else {
        // Notify user for manual approval
        notifySubscriptionDue(subscription)
    }
}
```

## Directory Operations

### Publish Profile

```swift
func publishProfile() async throws {
    // Profile is published to Pubky homeserver
    // Accessible at: pubky://<your-pubkey>/pub/pubky.app/profile.json
}
```

### Fetch Profile

```swift
func fetchProfile(pubkey: String) async throws -> PubkyProfile {
    return try await pubky_fetch_profile(pubkey: pubkey)
}
```

### Fetch Follows

```swift
func fetchFollows(pubkey: String) async throws -> [PubkyListItem] {
    return try await pubky_fetch_follows(pubkey: pubkey)
}
```

## Error Handling and Recovery

### Common Errors

```swift
enum PaykitErrorType {
    case notInitialized      // Call paykit_initialize first
    case networkError        // DHT/Homeserver unreachable
    case sessionExpired      // Re-authenticate needed
    case peerNotFound        // Pubkey not in directory
    case methodNotSupported  // Peer doesn't support this method
    case endpointUsed        // Endpoint needs rotation
}
```

### Recovery Strategies

```swift
func handlePaykitError(_ error: PaykitError) {
    switch error {
    case .notInitialized:
        // Re-initialize Paykit
        Task { try await initializePaykit() }
        
    case .sessionExpired:
        // Refresh session via Pubky-ring
        requestSessionRefresh()
        
    case .networkError:
        // Retry with exponential backoff
        scheduleRetry(delay: 1.0)
        
    case .endpointUsed:
        // Generate and publish new endpoint
        Task { try await rotateEndpoint(methodId: error.methodId) }
    }
}
```

## Endpoint Rotation

### Checking Rotation Need

Periodically check if endpoints need rotation:

```swift
func checkRotationNeeded() async throws -> [String] {
    let myPubkey = getMyPubkey()
    return try await paykit_check_rotation_needed(pubkey: myPubkey)
}

// Returns array of method IDs that need rotation, e.g., ["lightning", "onchain"]
```

### Rotating Endpoints

```swift
func rotateEndpoints() async throws {
    let needsRotation = try await checkRotationNeeded()
    
    for methodId in needsRotation {
        let newEndpoint = generateNewEndpoint(methodId: methodId)
        try await paykit_set_endpoint(methodId: methodId, endpoint: newEndpoint)
        print("Rotated endpoint for \(methodId)")
    }
}
```

### Background Task (iOS)

```swift
func registerRotationTask() {
    BGTaskScheduler.shared.register(
        forTaskWithIdentifier: "to.bitkit.rotationCheck",
        using: nil
    ) { task in
        Task {
            try await rotateEndpoints()
            task.setTaskCompleted(success: true)
        }
    }
}
```

## Cross-Device Authentication with Pubky-ring

### URL Scheme Communication

Bitkit can request authentication from Pubky-ring:

```swift
func requestPubkyAuth() {
    let authURL = URL(string: "pubkyring://auth?callback=bitkit://pubky-callback")!
    UIApplication.shared.open(authURL)
}

func handlePubkyCallback(url: URL) {
    guard let sessionData = url.queryParameter("session") else { return }
    storeSession(sessionData)
}
```

### Session Management

```swift
class SessionManager {
    func hasValidSession() -> Bool {
        guard let session = getStoredSession() else { return false }
        return !session.isExpired
    }
    
    func refreshSessionIfNeeded() async throws {
        guard let session = getStoredSession() else {
            throw PaykitError.notAuthenticated
        }
        
        if session.willExpireSoon {
            try await refreshSession()
        }
    }
}
```

## Testing

### Unit Test Example

```swift
class PaykitIntegrationTests: XCTestCase {
    func testDiscoverPaymentMethods() async throws {
        // Given
        let testPubkey = "o1gg96ewuojmopcjbz8895478wdtxtzzuxnfjjz8o8e77csa1ngo"
        
        // When
        let methods = try await paykit_get_supported_methods_for_key(pubkey: testPubkey)
        
        // Then
        XCTAssertNotNil(methods)
    }
    
    func testSmartCheckout() async throws {
        // Given
        let testPubkey = "o1gg96..."
        
        // When
        let result = try await paykit_smart_checkout(
            pubkey: testPubkey,
            preferredMethod: "lightning"
        )
        
        // Then
        XCTAssertFalse(result.endpoint.isEmpty)
    }
}
```

### Mock Transport for Testing

```kotlin
class MockPaykitService : PaykitService {
    private val mockEndpoints = mutableMapOf<String, String>()
    
    override suspend fun setEndpoint(methodId: String, endpoint: String) {
        mockEndpoints[methodId] = endpoint
    }
    
    override suspend fun getEndpoint(pubkey: String, methodId: String): String? {
        return mockEndpoints[methodId]
    }
}
```

## Performance Considerations

### Caching

Cache frequently accessed data:

```swift
class PaykitCache {
    private var methodCache: [String: PaykitSupportedMethods] = [:]
    private let cacheTimeout: TimeInterval = 300 // 5 minutes
    
    func getCachedMethods(for pubkey: String) -> PaykitSupportedMethods? {
        guard let entry = methodCache[pubkey],
              Date() < entry.expiresAt else {
            return nil
        }
        return entry.methods
    }
}
```

### Batch Operations

When processing multiple peers:

```swift
func discoverBatch(pubkeys: [String]) async throws -> [String: PaykitSupportedMethods] {
    return try await withThrowingTaskGroup(of: (String, PaykitSupportedMethods).self) { group in
        for pubkey in pubkeys {
            group.addTask {
                let methods = try await paykit_get_supported_methods_for_key(pubkey: pubkey)
                return (pubkey, methods)
            }
        }
        
        var results: [String: PaykitSupportedMethods] = [:]
        for try await (pubkey, methods) in group {
            results[pubkey] = methods
        }
        return results
    }
}
```

## Security Considerations

### Key Storage

- Store secret keys in Keychain (iOS) or Keystore (Android)
- Never log or expose keys
- Use biometric authentication for sensitive operations

### Session Security

- Sessions have expiration times
- Refresh sessions before expiry
- Clear sessions on logout

### Network Security

- All Pubky communication uses end-to-end encryption
- Noise protocol for interactive payments
- Verify peer identity before payments

## Troubleshooting

### Common Issues

1. **"Not initialized" error**
   - Call `paykit_initialize()` before other operations

2. **"Session expired" error**
   - Refresh session via Pubky-ring or re-authenticate

3. **"Peer not found" error**
   - Verify pubkey format (z-base-32 encoded)
   - Check network connectivity

4. **"Endpoint not found" error**
   - Peer may not have published this method
   - Try fallback methods

### Debug Logging

Enable debug logging during development:

```swift
#if DEBUG
func logPaykitOperation(_ operation: String, _ result: Any?) {
    print("[Paykit] \(operation): \(result ?? "nil")")
}
#endif
```

## Resources

- [Paykit Protocol Specification](../ARCHITECTURE_REVIEW.md)
- [Pubky Documentation](https://github.com/pubky/pubky-core)
- [Noise Protocol](https://noiseprotocol.org/)
- [FFI Patterns Guide](./FFI_PATTERNS.md)

