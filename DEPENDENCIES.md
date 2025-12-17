# Dependency Matrix

**Last Updated**: December 17, 2024

This document tracks all external dependencies across the Paykit/Bitkit ecosystem and their current versions.

---

## Repository Dependency Graph

```
bitkit-core (v0.2.5-pubky)
├── paykit-rs/paykit-lib (path: ../paykit-rs-master/paykit-lib)
│   ├── pubky (git: BitcoinErrorLog/pubky-core @ main)
│   └── pubky-testnet (git: BitcoinErrorLog/pubky-core @ main)
├── paykit-rs/paykit-interactive (path: ../paykit-rs-master/paykit-interactive)
├── pubky-noise (path: ../pubky-noise-main)
│   └── pubky (git: BitcoinErrorLog/pubky-core @ main)
└── pubky (git: BitcoinErrorLog/pubky-core @ main)

bitkit-ios (paykit-integration-complete branch)
└── BitkitCore (SPM: BitcoinErrorLog/bitkit-core @ v0.2.5-pubky)

bitkit-android (paykit-integration-complete branch)
└── bitkitcore (JNI: built from bitkit-core)

pubky-ring (main branch)
├── react-native-pubky (npm: @synonymdev/react-native-pubky)
└── pubky-noise-bindings (path: local FFI)
```

---

## Current Versions

### Core Libraries

| Library | Version | Repository | Status |
|---------|---------|------------|--------|
| **bitkit-core** | v0.2.5-pubky | [BitcoinErrorLog/bitkit-core](https://github.com/BitcoinErrorLog/bitkit-core) | ✅ Latest |
| **paykit-rs** | (workspace) | [synonymdev/paykit](https://github.com/synonymdev/paykit) | ✅ Working |
| **pubky-core** | main branch | [BitcoinErrorLog/pubky-core](https://github.com/BitcoinErrorLog/pubky-core) | ⚠️ Untagged |
| **pubky-noise** | v1.0.0 | [BitcoinErrorLog/pubky-noise](https://github.com/BitcoinErrorLog/pubky-noise) | ✅ Tagged |

### Mobile Apps

| App | Branch | Core Version | Status |
|-----|--------|--------------|--------|
| **bitkit-ios** | paykit-integration-complete | v0.2.5-pubky | ✅ Current |
| **bitkit-android** | paykit-integration-complete | v0.2.5-pubky | ✅ Current |
| **pubky-ring** | main | (local build) | ✅ Working |

---

## Dependency Details

### bitkit-core Dependencies

**From `Cargo.toml`:**
```toml
[dependencies]
uniffi = { version = "0.29.4", features = [ "cli", "bindgen" ] }
paykit-lib = { path = "../paykit-rs-master/paykit-lib", features = ["pubky"] }
paykit-interactive = { path = "../paykit-rs-master/paykit-interactive" }
pubky-noise = { path = "../pubky-noise-main", features = ["pubky-sdk"] }
pubky = { git = "https://github.com/BitcoinErrorLog/pubky-core" }

# ... other dependencies (bitcoin, lightning-invoice, etc.)
```

**Git Dependencies:**
- `pubky`: Uses `main` branch from BitcoinErrorLog/pubky-core
- Status: ⚠️ Should pin to version tag

---

### paykit-rs Dependencies

**From `paykit-lib/Cargo.toml`:**
```toml
[dependencies.pubky]
git = "https://github.com/BitcoinErrorLog/pubky-core"
optional = true

[dev-dependencies]
pubky-testnet = { git = "https://github.com/BitcoinErrorLog/pubky-core", branch = "main" }
```

**Status**: ⚠️ Should pin to version tag

---

### pubky-noise Dependencies

**From `Cargo.toml`:**
```toml
[dependencies]
pubky = { git = "https://github.com/BitcoinErrorLog/pubky-core", optional = true }
```

**Status**: ⚠️ Should pin to version tag

---

## Known Issues

### 1. Untagged pubky-core Dependency 🔴

**Problem**: All repos depend on `main` branch of pubky-core, making it hard to track what version is deployed.

**Impact**: 
- Version drift across repos
- Hard to reproduce builds
- Breaking changes can propagate unexpectedly

**Solution**:
1. Tag pubky-core with v0.6.0
2. Update all repos to use:
   ```toml
   pubky = { git = "https://github.com/BitcoinErrorLog/pubky-core", tag = "v0.6.0" }
   ```

---

### 2. Path Dependencies for Local Development 🟡

**Current**: Using path dependencies for local development:
```toml
paykit-lib = { path = "../paykit-rs-master/paykit-lib" }
```

**Pros**:
- Easy local iteration
- No need to publish intermediate versions

**Cons**:
- Not reproducible on other machines
- Can't build from Git checkout alone

**Recommendation**: 
- Keep path dependencies for development
- Document workspace structure in README
- For releases, consider git dependencies or publishing to crates.io

---

## Version Update Process

### When to Update

Update dependencies when:
1. Security vulnerability is fixed
2. New feature is needed
3. Bug fix is required
4. Regular maintenance (quarterly)

### How to Update

#### 1. Update pubky-core

```bash
# In each repo that uses pubky
cd bitkit-core
vim Cargo.toml
# Change: git = "...", branch = "main"
# To:     git = "...", tag = "v0.6.1"

cargo update -p pubky
cargo test --lib
```

#### 2. Update bitkit-core in Mobile Apps

**iOS:**
```bash
cd bitkit-ios
# Update Package.swift or Xcode project to new release
# File → Packages → Update to Latest Package Versions
```

**Android:**
```bash
cd bitkit-android
# Rebuild bitkit-core
cd ../bitkit-core
./build_android.sh
# Copy .so files to bitkit-android/app/src/main/jniLibs/
```

#### 3. Test Integration

```bash
# Run integration tests
cd bitkit-core
cargo test --test '*'

# Build iOS
./build_ios.sh

# Build Android  
./build_android.sh

# Test mobile apps
# (Manual testing for now, automated in Phase 2)
```

#### 4. Update This Document

```bash
# Update version numbers in this file
# Commit with message: "chore: update dependencies to pubky-core v0.6.1"
```

---

## Dependency Licenses

| Library | License | Notes |
|---------|---------|-------|
| uniffi | MPL-2.0 | Mozilla Public License |
| paykit-rs | MIT | Open source |
| pubky-core | MIT | Open source |
| pubky-noise | MIT | Open source |
| bitcoin | CC0-1.0 | Public domain |
| lightning | Apache-2.0 | Open source |
| ldk-node | Apache-2.0 | Open source |

**Overall**: All dependencies are permissive open-source licenses compatible with commercial use.

---

## Version History

### v0.2.5-pubky (December 17, 2024)
- Fixed parse_public_key recursion
- Completed Android ViewModels
- Added endpoint health checking
- All repos using git pubky-core @ main

### v0.2.4-pubky (December 17, 2024)
- Fixed test fixtures with seen_at
- Minor improvements

### v0.2.3-pubky (December 17, 2024)
- Added seenAt tracking
- Added activity query FFIs
- Unified pubky git deps

### v0.2.2-pubky (December 16, 2024)
- Initial pubky SDK integration
- Profile type consolidation

---

## Future Improvements

### Phase 1: Version Pinning (This Week)
- [ ] Tag pubky-core v0.6.0
- [ ] Update all repos to use version tags
- [ ] Document update process

### Phase 2: Automation (Next Week)
- [ ] Script to check for dependency updates
- [ ] Automated testing of dependency updates
- [ ] Dependabot configuration

### Phase 3: Publishing (Future)
- [ ] Consider publishing bitkit-core to crates.io
- [ ] Semantic versioning strategy
- [ ] Changelog automation

---

## Contact

For dependency-related questions:
- Architecture: See `ARCHITECTURE_REVIEW.md`
- Issues: Open GitHub issue in respective repo
- Updates: Follow this document

**Maintainers**:
- bitkit-core: BitcoinErrorLog org
- paykit-rs: Synonym team
- pubky-core: BitcoinErrorLog org
- pubky-noise: BitcoinErrorLog org

