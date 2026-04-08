//! Command authentication and message integrity for space communication systems.
//!
//! Provides HMAC-SHA256-based signing and verification primitives that protect
//! command uplinks and telemetry downlinks from tampering and replay attacks.
//!
//! # Design Constraints
//! - No heap allocation; operates on fixed-size arrays compatible with `no_std`.
//! - HMAC-SHA256 digest size is 32 bytes; callers must allocate accordingly.
//! - Key material is caller-supplied; this module never stores secret keys.
//!
//! # Requirements Traceability
//! - REQ-SF-001: Command validation and confirmation requirements
//! - REQ-SF-002: Override protection for critical safety functions
//!
//! # Standards References
//! - FIPS PUB 198-1: The Keyed-Hash Message Authentication Code (HMAC)
//! - FIPS PUB 180-4: Secure Hash Standard (SHA-2 family)
//! - DoD 8570.01-M: Information Assurance Workforce Improvement Program

use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};

use crate::error::{Result, SpaceCommError, CryptoOperation};

/// HMAC-SHA256 output length in bytes.
pub const DIGEST_LEN: usize = 32;

/// Authentication tag produced by `CommandAuthenticator::sign`.
///
/// A 32-byte HMAC-SHA256 digest that binds a message to a shared secret key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthTag(pub [u8; DIGEST_LEN]);

impl AuthTag {
    /// Create an `AuthTag` from a raw 32-byte array.
    pub const fn new(bytes: [u8; DIGEST_LEN]) -> Self {
        Self(bytes)
    }

    /// Return a reference to the raw digest bytes.
    pub const fn as_bytes(&self) -> &[u8; DIGEST_LEN] {
        &self.0
    }
}

/// HMAC-SHA256 command authenticator.
///
/// - **ID**: MOD-SEC-001
/// - **Requirement**: Authenticate every command before processing to prevent
///   injection of forged commands into the priority queue (REQ-SF-001).
/// - **Purpose**: Bind commands to a shared symmetric key so that an adversary
///   who intercepts the RF link cannot construct accepted commands.
/// - **Rationale**: HMAC-SHA256 provides 128-bit collision resistance and is
///   approved for use in government communications under FIPS 198-1.
///   It operates on fixed-size data without heap allocation.
/// - **Assumptions**: Both sender and receiver share the same `key` out-of-band.
///   Key rotation policy is the responsibility of the calling system.
/// - **Failure Modes**: Sign is infallible for valid key sizes (1–64 bytes).
///   Verify returns `Err` on invalid key or `Ok(false)` on tag mismatch.
/// - **Constraints**: No heap allocation; key length 1–64 bytes.
/// - **References**: FIPS PUB 198-1; FIPS PUB 180-4.
pub struct CommandAuthenticator;

impl CommandAuthenticator {
    /// Produce an HMAC-SHA256 authentication tag over `message` using `key`.
    ///
    /// - **ID**: FN-SEC-001
    /// - **Requirement**: Generate a 32-byte HMAC-SHA256 tag that a receiver
    ///   can verify with the same key (REQ-SF-001).
    /// - **Inputs**:
    ///   - `key`: Shared secret; must be 1–64 bytes; longer keys are hashed internally.
    ///   - `message`: Arbitrary byte slice to authenticate; may be empty.
    /// - **Outputs**: `Ok(AuthTag)` on success.
    /// - **Preconditions**: `!key.is_empty()`.
    /// - **Postconditions**: Return value is deterministic for identical `(key, message)`.
    /// - **Failure Modes**: Empty key → `Err(CryptographicError)`.
    /// - **Side Effects**: None — pure function with no state mutation.
    /// - **Constraints**: O(|message|) time; O(1) additional space.
    /// - **Verification**: `sign(k, m)` then `verify(k, m, tag)` must return `Ok(true)`.
    /// - **References**: FIPS PUB 198-1 §3.
    pub fn sign(key: &[u8], message: &[u8]) -> Result<AuthTag> {
        if key.is_empty() {
            return Err(SpaceCommError::CryptographicError {
                operation: CryptoOperation::Signing,
                details: "HMAC key must not be empty",
            });
        }

        let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| {
            SpaceCommError::CryptographicError {
                operation: CryptoOperation::Signing,
                details: "Invalid HMAC key length",
            }
        })?;
        mac.update(message);
        let result = mac.finalize();
        let bytes: [u8; DIGEST_LEN] = result.into_bytes().into();
        Ok(AuthTag::new(bytes))
    }

    /// Verify that `tag` is a valid HMAC-SHA256 over `message` under `key`.
    ///
    /// - **ID**: FN-SEC-002
    /// - **Requirement**: Accept a command only when its authentication tag
    ///   matches the locally computed HMAC, preventing forged command injection.
    /// - **Inputs**:
    ///   - `key`: Shared secret; must match the key used during `sign`.
    ///   - `message`: The byte slice that was originally signed.
    ///   - `tag`: The `AuthTag` received alongside the message.
    /// - **Outputs**: `Ok(true)` — tag valid; `Ok(false)` — tag invalid (reject);
    ///   `Err` — key invalid.
    /// - **Preconditions**: `!key.is_empty()`.
    /// - **Postconditions**: Returns `Ok(true)` iff `tag == sign(key, message)`.
    /// - **Failure Modes**: Timing-safe comparison prevents side-channel leakage.
    ///   Invalid key length → `Err(CryptographicError)`.
    /// - **Side Effects**: None.
    /// - **Constraints**: O(|message|) time; comparison is constant-time.
    /// - **References**: FIPS PUB 198-1 §3; NIST SP 800-107 Rev.1.
    pub fn verify(key: &[u8], message: &[u8], tag: &AuthTag) -> Result<bool> {
        if key.is_empty() {
            return Err(SpaceCommError::CryptographicError {
                operation: CryptoOperation::Verification,
                details: "HMAC key must not be empty",
            });
        }

        let mut mac = Hmac::<Sha256>::new_from_slice(key).map_err(|_| {
            SpaceCommError::CryptographicError {
                operation: CryptoOperation::Verification,
                details: "Invalid HMAC key length",
            }
        })?;
        mac.update(message);

        // `verify_slice` performs a constant-time comparison to prevent
        // timing-based side-channel attacks.
        Ok(mac.verify_slice(tag.as_bytes()).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify_roundtrip() {
        let key = b"mission-auth-key-32bytes-exactly";
        let message = b"EMERGENCY_ABORT:0xDEADBEEF";
        let tag = CommandAuthenticator::sign(key, message).unwrap();
        assert!(CommandAuthenticator::verify(key, message, &tag).unwrap());
    }

    #[test]
    fn test_verify_rejects_tampered_message() {
        let key = b"mission-auth-key-32bytes-exactly";
        let message = b"EMERGENCY_ABORT:0xDEADBEEF";
        let tag = CommandAuthenticator::sign(key, message).unwrap();
        // Tampered message
        let bad = b"EMERGENCY_ABORT:0x00000000";
        assert!(!CommandAuthenticator::verify(key, bad, &tag).unwrap());
    }

    #[test]
    fn test_verify_rejects_wrong_key() {
        let key = b"mission-auth-key-32bytes-exactly";
        let wrong_key = b"wrong-key-32-bytes-long-here-xxx";
        let message = b"ORBIT_UPDATE";
        let tag = CommandAuthenticator::sign(key, message).unwrap();
        assert!(!CommandAuthenticator::verify(wrong_key, message, &tag).unwrap());
    }

    #[test]
    fn test_empty_key_returns_error() {
        let result = CommandAuthenticator::sign(b"", b"data");
        assert!(result.is_err());
    }
}
