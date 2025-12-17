# Generating Mobile Bindings for Bitkit Core

This document provides instructions for generating the FFI bindings for iOS and Android, including the new Paykit Interactive functionality.

## Prerequisites

*   **Rust**: Latest stable version (`rustup update stable`).
*   **Targets**:
    *   iOS: `aarch64-apple-ios`, `aarch64-apple-ios-sim`
    *   Android: `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, `x86_64-linux-android`
*   **Tools**:
    *   `cargo-ndk` (for Android)
    *   `uniffi-bindgen` (installed via scripts)
    *   Android NDK (environment variable `ANDROID_NDK_HOME` set)
    *   Xcode (for iOS)

## iOS Bindings

To generate the Swift bindings and `BitkitCore.xcframework`:

```bash
./build_ios.sh
```

**Output**:
*   `bindings/ios/bitkitcore.swift`: Swift source file.
*   `bindings/ios/BitkitCore.xcframework`: Binary framework.

## Android Bindings

To generate the Kotlin bindings and compiled `.so` libraries:

```bash
./build_android.sh
```

**Output**:
*   `bindings/android/lib/src/main/kotlin/com/synonym/bitkitcore/`: Kotlin source files (`bitkitcore.android.kt`, `bitkitcore.common.kt`).
*   `bindings/android/lib/src/main/jniLibs/`: Compiled native libraries.

## New Paykit Functionality

The bindings will now include the `PaykitInteractive` class/object, which exposes:

*   `constructor(db_path: String, secret_key_hex: String)`
*   `initiate_payment(host: String, port: u16, peer_pubkey: String, receipt: PaykitReceiptFfi)`

### Example Usage (Swift)

```swift
let paykit = try PaykitInteractive(dbPath: "path/to/activity.db", secretKeyHex: userSecret)
let finalReceipt = try await paykit.initiatePayment(
    host: "192.168.1.5",
    port: 1234,
    peerPubkey: peerPubkeyString,
    receipt: provisionalReceipt
)
```

### Example Usage (Kotlin)

```kotlin
val paykit = PaykitInteractive(dbPath, userSecret)
val finalReceipt = paykit.initiatePayment(
    host = "192.168.1.5",
    port = 1234.toUShort(),
    peerPubkey = peerPubkeyString,
    receipt = provisionalReceipt
)
```

