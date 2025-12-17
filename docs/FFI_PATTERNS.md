# FFI Patterns Guide

This document describes patterns for working with the UniFFI-based FFI layer in bitkit-core.

## Overview

bitkit-core uses [UniFFI](https://mozilla.github.io/uniffi-rs/) to generate bindings for Swift (iOS) and Kotlin (Android). The core library is written in Rust, and UniFFI generates the necessary glue code.

## Setup

### Cargo.toml Dependencies

```toml
[dependencies]
uniffi = { version = "0.29.4", features = ["cli", "bindgen"] }
```

### Crate Configuration

```rust
// src/lib.rs
uniffi::setup_scaffolding!();
```

## Adding New FFI Functions

### Step 1: Define the Function

Add the `#[uniffi::export]` attribute to expose a function:

```rust
#[uniffi::export]
pub fn my_function(param: String) -> Result<MyType, MyError> {
    // Implementation
}
```

### Step 2: Define Types

Types must be FFI-compatible. Use UniFFI record macros:

```rust
#[derive(uniffi::Record)]
pub struct MyType {
    pub field1: String,
    pub field2: u64,
    pub optional_field: Option<String>,
}
```

### Step 3: Define Errors

Use thiserror with UniFFI:

```rust
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum MyError {
    #[error("Invalid input: {error_details}")]
    InvalidInput { error_details: String },
    
    #[error("Network error: {error_details}")]
    NetworkError { error_details: String },
}
```

### Step 4: Async Functions

For async operations, use the tokio runtime:

```rust
#[uniffi::export]
pub async fn my_async_function(param: String) -> Result<MyType, MyError> {
    let rt = ensure_runtime();
    rt.spawn(async move {
        // Async implementation
    })
    .await
    .unwrap()
}
```

## Error Handling Patterns

### Result Types

Always return `Result<T, E>` for fallible operations:

```rust
#[uniffi::export]
pub fn parse_invoice(invoice: String) -> Result<Invoice, DecodingError> {
    // Use ? operator for clean error propagation
    let parsed = Bolt11Invoice::from_str(&invoice)?;
    Ok(Invoice::from(parsed))
}
```

### Error Conversion

Implement `From` for error type conversions:

```rust
impl From<bitcoin::Error> for PaykitError {
    fn from(e: bitcoin::Error) -> Self {
        PaykitError::Bitcoin { error_details: e.to_string() }
    }
}
```

## Type Conversion Best Practices

### Vec<u8> for Binary Data

Use `Vec<u8>` for binary data (keys, hashes, signatures):

```rust
#[derive(uniffi::Record)]
pub struct Transaction {
    pub tx_id: String,           // Hex-encoded for display
    pub raw_tx: Vec<u8>,         // Raw binary data
    pub signature: Vec<u8>,      // Binary signature
}
```

### String for Human-Readable Data

Use `String` for identifiers and formatted data:

```rust
#[derive(uniffi::Record)]
pub struct Address {
    pub address: String,         // Human-readable address
    pub derivation_path: String, // BIP32 path like "m/84'/0'/0'/0/0"
}
```

### Optional Fields

Use `Option<T>` for fields that may not be present:

```rust
#[derive(uniffi::Record)]
pub struct Invoice {
    pub amount_satoshis: Option<u64>,  // Zero-amount invoices have None
    pub description: Option<String>,   // Not all invoices have descriptions
}
```

### HashMap for Dynamic Data

Use `HashMap<String, String>` for flexible key-value data:

```rust
#[derive(uniffi::Record)]
pub struct Metadata {
    pub fields: HashMap<String, String>,
}
```

## Async/Await Patterns

### Runtime Management

The crate maintains a global tokio runtime:

```rust
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn ensure_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create Tokio runtime"))
}
```

### Spawning Tasks

Wrap async work in `rt.spawn`:

```rust
#[uniffi::export]
pub async fn fetch_data(url: String) -> Result<Data, NetworkError> {
    let rt = ensure_runtime();
    rt.spawn(async move {
        let response = reqwest::get(&url).await?;
        Ok(response.json().await?)
    })
    .await
    .unwrap_or_else(|e| Err(NetworkError::Runtime { 
        error_details: e.to_string() 
    }))
}
```

### Blocking Operations

For sync code that needs to call async code, use `block_on`:

```rust
pub fn sync_wrapper() -> Result<Data, Error> {
    let rt = ensure_runtime();
    rt.block_on(async_implementation())
}
```

**Warning**: Never call `block_on` from within an async context (nested runtime error).

## Platform-Specific Considerations

### iOS (Swift)

- Memory is managed automatically via ARC
- Error types are mapped to Swift enums
- Async functions become Swift async functions
- Use `DispatchQueue.main.async` for UI updates after FFI calls

Example Swift usage:

```swift
Task {
    do {
        let result = try await decode(invoice: invoiceString)
        // Handle result
    } catch {
        // Handle error
    }
}
```

### Android (Kotlin)

- JNI handles memory management
- Error types are mapped to Kotlin sealed classes
- Async functions become suspend functions
- Use Coroutines for async operations

Example Kotlin usage:

```kotlin
viewModelScope.launch {
    try {
        val result = decode(invoiceString)
        // Handle result
    } catch (e: DecodingException) {
        // Handle error
    }
}
```

## Testing FFI Functions

### Unit Tests

Test the Rust implementation directly:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let result = validate_bitcoin_address("bc1q...".to_string());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = fetch_data("https://example.com".to_string()).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Create tests that exercise the full FFI path:

```rust
// tests/integration/ffi_tests.rs
#[test]
fn test_ffi_roundtrip() {
    // Initialize database
    init_db("/tmp/test".to_string()).unwrap();
    
    // Call FFI function
    let activity = create_test_activity();
    upsert_activity(activity.clone()).unwrap();
    
    // Verify
    let fetched = get_activity_by_id(activity.id).unwrap();
    assert_eq!(fetched, Some(activity));
}
```

## Debugging Tips

### Logging

Use structured logging in FFI functions:

```rust
#[uniffi::export]
pub fn my_function(input: String) -> Result<Output, Error> {
    eprintln!("my_function called with: {}", input);
    // Implementation
}
```

### Error Messages

Include context in error messages:

```rust
Err(MyError::InvalidInput {
    error_details: format!("Expected hex string, got: {}", input.chars().take(20).collect::<String>())
})
```

### Build Verification

After changes, always verify:

```bash
# Check Rust compilation
cargo check --lib

# Run tests
cargo test --lib

# Build iOS bindings
./build_ios.sh

# Build Android bindings
./build_android.sh
```

## Common Pitfalls

### 1. Nested Runtime

**Wrong:**
```rust
#[tokio::test]
async fn test() {
    init_db("path".to_string()); // Uses block_on internally!
}
```

**Right:**
```rust
#[test]
fn test() {
    init_db("path".to_string()).unwrap();
}
```

### 2. Missing Export Attribute

**Wrong:**
```rust
pub fn my_function() -> Result<(), Error> { }
```

**Right:**
```rust
#[uniffi::export]
pub fn my_function() -> Result<(), Error> { }
```

### 3. Non-FFI-Compatible Types

**Wrong:**
```rust
#[uniffi::export]
pub fn my_function() -> Box<dyn Trait> { } // Trait objects not supported
```

**Right:**
```rust
#[uniffi::export]
pub fn my_function() -> ConcreteType { }
```

### 4. Panics in FFI

Never panic in FFI functions. Always return Result:

**Wrong:**
```rust
#[uniffi::export]
pub fn my_function(input: String) -> String {
    input.parse().unwrap() // Panic!
}
```

**Right:**
```rust
#[uniffi::export]
pub fn my_function(input: String) -> Result<String, ParseError> {
    input.parse().map_err(|e| ParseError { details: e.to_string() })
}
```

## Build Scripts

### iOS Build

```bash
./build_ios.sh
```

This generates:
- `bindings/swift/bitkitcore.swift`
- `target/aarch64-apple-ios/release/libbitkitcore.a`

### Android Build

```bash
./build_android.sh
```

This generates:
- `bindings/kotlin/bitkitcore/bitkitcore.kt`
- `target/aarch64-linux-android/release/libbitkitcore.so`
- `target/armv7-linux-androideabi/release/libbitkitcore.so`
- `target/x86_64-linux-android/release/libbitkitcore.so`

## Resources

- [UniFFI Book](https://mozilla.github.io/uniffi-rs/)
- [UniFFI Examples](https://github.com/aspect-dev/aspect-workflows-examples/tree/main/examples/uniffi)
- [Rust Async Book](https://rust-lang.github.io/async-book/)

