//! Token-based authentication for the server
//!
//! Generates and validates authentication tokens for secure access.

use rand::Rng;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AuthError {
    IoError(io::Error),
    InvalidToken,
    TokenNotFound,
}

impl From<io::Error> for AuthError {
    fn from(err: io::Error) -> Self {
        AuthError::IoError(err)
    }
}

pub struct TokenAuth {
    token: String,
    token_path: PathBuf,
}

impl TokenAuth {
    /// Create a new TokenAuth, loading or generating a token
    ///
    /// Priority order:
    /// 1. Environment variable TITI_TOKEN (if set)
    /// 2. Existing token file (~/.titi/token)
    /// 3. Generate new token and save to file
    pub fn new() -> Result<Self, AuthError> {
        let token_path = Self::get_token_path()?;

        // First, check if environment variable is set
        let token = if let Ok(env_token) = std::env::var("TITI_TOKEN") {
            // Use token from environment variable (for testing or custom deployments)
            env_token
        } else if token_path.exists() {
            // Try to load existing token from file
            fs::read_to_string(&token_path)?
        } else {
            // Generate new token
            let new_token = Self::generate_token();

            // Create directory if it doesn't exist
            if let Some(parent) = token_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write token to file with restricted permissions
            fs::write(&token_path, &new_token)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&token_path, fs::Permissions::from_mode(0o600))?;
            }

            new_token
        };

        Ok(Self { token, token_path })
    }

    /// Load token from environment variable or file
    pub fn load() -> Result<String, AuthError> {
        // First try environment variable
        if let Ok(token) = std::env::var("TITI_TOKEN") {
            return Ok(token);
        }

        // Then try file
        let token_path = Self::get_token_path()?;
        if token_path.exists() {
            Ok(fs::read_to_string(&token_path)?)
        } else {
            Err(AuthError::TokenNotFound)
        }
    }

    /// Validate a token against the stored token
    pub fn validate(&self, token: &str) -> bool {
        self.token == token
    }

    /// Get the authentication token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the path to the token file
    pub fn token_path(&self) -> &PathBuf {
        &self.token_path
    }

    /// Create TokenAuth from a specific token (for testing)
    #[cfg(test)]
    pub fn from_token(token: String) -> Result<Self, AuthError> {
        Ok(Self {
            token,
            token_path: PathBuf::from("/tmp/titi_test_token"),
        })
    }

    /// Generate a random 64-character token
    fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        (0..64)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                match idx {
                    0..=9 => (b'0' + idx) as char,
                    10..=35 => (b'a' + (idx - 10)) as char,
                    36..=61 => (b'A' + (idx - 36)) as char,
                    _ => unreachable!(),
                }
            })
            .collect()
    }

    /// Get the path to the token file (~/.titi/token)
    fn get_token_path() -> Result<PathBuf, AuthError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AuthError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find home directory"
            )))?;

        Ok(home.join(".titi").join("token"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let token = TokenAuth::generate_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_validate_token() {
        let auth = TokenAuth {
            token: "test_token_123".to_string(),
            token_path: PathBuf::new(),
        };

        assert!(auth.validate("test_token_123"));
        assert!(!auth.validate("wrong_token"));
    }
}
