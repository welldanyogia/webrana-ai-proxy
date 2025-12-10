//! AES-256-GCM encryption utilities for API key storage.
//! 
//! Requirements: 3.1, 3.2 - AES-256-GCM with unique 12-byte IV per encryption

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use std::env;

/// Encrypted data structure for database storage
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub iv: [u8; 12],
    pub auth_tag: [u8; 16],
}

/// Encryption error
#[derive(Debug)]
pub enum EncryptionError {
    InvalidKey,
    EncryptionFailed,
    DecryptionFailed,
    MissingMasterKey,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::InvalidKey => write!(f, "Invalid encryption key"),
            EncryptionError::EncryptionFailed => write!(f, "Encryption failed"),
            EncryptionError::DecryptionFailed => write!(f, "Decryption failed"),
            EncryptionError::MissingMasterKey => write!(f, "Master encryption key not configured"),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Encryption utilities
pub struct EncryptionUtils {
    cipher: Aes256Gcm,
}

impl EncryptionUtils {
    /// Create new encryption utils from environment variable
    pub fn from_env() -> Result<Self, EncryptionError> {
        let key_b64 = env::var("MASTER_ENCRYPTION_KEY")
            .map_err(|_| EncryptionError::MissingMasterKey)?;
        
        let key_bytes = BASE64.decode(&key_b64)
            .map_err(|_| EncryptionError::InvalidKey)?;
        
        if key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidKey);
        }
        
        let key: [u8; 32] = key_bytes.try_into()
            .map_err(|_| EncryptionError::InvalidKey)?;
        
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|_| EncryptionError::InvalidKey)?;
        
        Ok(Self { cipher })
    }

    /// Create encryption utils from raw key bytes (for testing)
    #[cfg(test)]
    pub fn from_key(key: &[u8; 32]) -> Result<Self, EncryptionError> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| EncryptionError::InvalidKey)?;
        Ok(Self { cipher })
    }

    /// Encrypt plaintext with unique IV
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, EncryptionError> {
        // Generate unique 12-byte IV
        let mut iv = [0u8; 12];
        OsRng.fill_bytes(&mut iv);
        let nonce = Nonce::from_slice(&iv);
        
        // Encrypt
        let ciphertext_with_tag = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| EncryptionError::EncryptionFailed)?;
        
        // Split ciphertext and auth tag (last 16 bytes)
        let tag_start = ciphertext_with_tag.len() - 16;
        let ciphertext = ciphertext_with_tag[..tag_start].to_vec();
        let auth_tag: [u8; 16] = ciphertext_with_tag[tag_start..]
            .try_into()
            .map_err(|_| EncryptionError::EncryptionFailed)?;
        
        Ok(EncryptedData {
            ciphertext,
            iv,
            auth_tag,
        })
    }

    /// Decrypt ciphertext
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<String, EncryptionError> {
        let nonce = Nonce::from_slice(&encrypted.iv);
        
        // Combine ciphertext and auth tag
        let mut ciphertext_with_tag = encrypted.ciphertext.clone();
        ciphertext_with_tag.extend_from_slice(&encrypted.auth_tag);
        
        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext_with_tag.as_ref())
            .map_err(|_| EncryptionError::DecryptionFailed)?;
        
        String::from_utf8(plaintext)
            .map_err(|_| EncryptionError::DecryptionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const TEST_KEY: [u8; 32] = [0u8; 32];

    fn test_utils() -> EncryptionUtils {
        EncryptionUtils::from_key(&TEST_KEY).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let utils = test_utils();
        
        let plaintext = "sk-test-api-key-12345";
        let encrypted = utils.encrypt(plaintext).unwrap();
        let decrypted = utils.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_unique_iv_per_encryption() {
        let utils = test_utils();
        
        let plaintext = "same-plaintext";
        let encrypted1 = utils.encrypt(plaintext).unwrap();
        let encrypted2 = utils.encrypt(plaintext).unwrap();
        
        assert_ne!(encrypted1.iv, encrypted2.iv);
    }

    // Property Test 1: Encryption Round-Trip Consistency
    // Validates: Requirements 4.5 - Any encrypted API key can be decrypted to original
    proptest! {
        #[test]
        fn prop_encryption_roundtrip(plaintext in "[a-zA-Z0-9_-]{1,200}") {
            let utils = test_utils();
            let encrypted = utils.encrypt(&plaintext).unwrap();
            let decrypted = utils.decrypt(&encrypted).unwrap();
            prop_assert_eq!(plaintext, decrypted);
        }
    }

    // Property Test 8: Unique IV Per Encryption
    // Validates: Requirements 3.1 - Each encryption generates unique IV
    proptest! {
        #[test]
        fn prop_unique_iv_per_encryption(plaintext in "[a-zA-Z0-9]{10,50}") {
            let utils = test_utils();
            let encrypted1 = utils.encrypt(&plaintext).unwrap();
            let encrypted2 = utils.encrypt(&plaintext).unwrap();
            // IVs must be different even for same plaintext
            prop_assert_ne!(encrypted1.iv, encrypted2.iv);
            // Ciphertexts should also differ due to different IVs
            prop_assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
        }
    }

    // Property Test: Ciphertext differs from plaintext
    proptest! {
        #[test]
        fn prop_ciphertext_differs_from_plaintext(plaintext in "[a-zA-Z0-9]{10,100}") {
            let utils = test_utils();
            let encrypted = utils.encrypt(&plaintext).unwrap();
            // Ciphertext should not contain plaintext
            prop_assert_ne!(encrypted.ciphertext, plaintext.as_bytes());
        }
    }

    // Property Test: Auth tag is always 16 bytes
    proptest! {
        #[test]
        fn prop_auth_tag_size(plaintext in ".{1,500}") {
            let utils = test_utils();
            let encrypted = utils.encrypt(&plaintext).unwrap();
            prop_assert_eq!(encrypted.auth_tag.len(), 16);
            prop_assert_eq!(encrypted.iv.len(), 12);
        }
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let utils = test_utils();
        let plaintext = "sk-secret-key";
        let mut encrypted = utils.encrypt(plaintext).unwrap();
        
        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }
        
        // Decryption should fail
        assert!(utils.decrypt(&encrypted).is_err());
    }

    #[test]
    fn test_tampered_auth_tag_fails() {
        let utils = test_utils();
        let plaintext = "sk-secret-key";
        let mut encrypted = utils.encrypt(plaintext).unwrap();
        
        // Tamper with auth tag
        encrypted.auth_tag[0] ^= 0xFF;
        
        // Decryption should fail
        assert!(utils.decrypt(&encrypted).is_err());
    }
}
