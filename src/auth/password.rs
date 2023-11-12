use pbkdf2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct HashPasswordError {
    details: String,
}

impl HashPasswordError {
    fn new(msg: &str) -> HashPasswordError {
        HashPasswordError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for HashPasswordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for HashPasswordError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub fn hash(password: &str) -> Result<String, HashPasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| HashPasswordError::new(&e.to_string()))?
        .to_string();
    Ok(hash)
}

pub fn verify(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(parsed_hash) => parsed_hash,
        Err(_) => return false,
    };

    Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let password = "password";
        let hash = hash(password).unwrap();
        assert_ne!(password, hash);
    }

    #[test]
    fn test_verify() {
        let password = "password";
        let hash = hash(password).unwrap();
        assert!(verify(password, &hash));
    }
}
