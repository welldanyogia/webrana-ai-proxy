//! Argon2id password hashing utilities.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Password hashing error
#[derive(Debug)]
pub enum PasswordError {
    HashingFailed,
    VerificationFailed,
    InvalidHash,
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordError::HashingFailed => write!(f, "Password hashing failed"),
            PasswordError::VerificationFailed => write!(f, "Password verification failed"),
            PasswordError::InvalidHash => write!(f, "Invalid password hash format"),
        }
    }
}

impl std::error::Error for PasswordError {}

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| PasswordError::HashingFailed)?;
    
    Ok(hash.to_string())
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| PasswordError::InvalidHash)?;
    
    let argon2 = Argon2::default();
    
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_hash_password_not_plaintext() {
        let password = "my-secure-password";
        let hash = hash_password(password).unwrap();
        
        assert_ne!(password, hash);
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_correct_password() {
        let password = "my-secure-password";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let password = "my-secure-password";
        let hash = hash_password(password).unwrap();
        
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    // Property Test 2: Password Hashing Security
    // Validates: Requirements 1.2, 6.2 - Passwords hashed with Argon2id
    // Note: Limited cases due to Argon2 being intentionally slow
    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(5))]
        #[test]
        fn prop_password_hash_roundtrip(password in "[a-zA-Z0-9!@#$%^&*]{8,32}") {
            let hash = hash_password(&password).unwrap();
            // Correct password verifies
            prop_assert!(verify_password(&password, &hash).unwrap());
        }
    }

    // Property Test: Hash is never plaintext
    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(5))]
        #[test]
        fn prop_hash_not_plaintext(password in "[a-zA-Z0-9]{8,32}") {
            let hash = hash_password(&password).unwrap();
            // Hash should never equal plaintext
            prop_assert_ne!(&hash, &password);
            // Hash should start with Argon2 identifier
            prop_assert!(hash.starts_with("$argon2"));
        }
    }

    // Property Test: Different salts produce different hashes
    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(3))]
        #[test]
        fn prop_unique_salt_per_hash(password in "[a-zA-Z0-9]{8,16}") {
            let hash1 = hash_password(&password).unwrap();
            let hash2 = hash_password(&password).unwrap();
            // Same password should produce different hashes (different salts)
            prop_assert_ne!(&hash1, &hash2);
            // But both should verify correctly
            prop_assert!(verify_password(&password, &hash1).unwrap());
            prop_assert!(verify_password(&password, &hash2).unwrap());
        }
    }

    // Property Test: Wrong password never verifies
    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(3))]
        #[test]
        fn prop_wrong_password_fails(
            password in "[a-zA-Z0-9]{8,16}",
            wrong in "[a-zA-Z0-9]{8,16}"
        ) {
            prop_assume!(password != wrong);
            let hash = hash_password(&password).unwrap();
            prop_assert!(!verify_password(&wrong, &hash).unwrap());
        }
    }
}
