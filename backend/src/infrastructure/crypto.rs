use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// AES-256-GCM encryptor for database connection passwords.
#[derive(Clone)]
pub struct Encryptor {
    key: [u8; 32],
}

impl Encryptor {
    /// Create an Encryptor from the `ENCRYPTION_KEY` environment variable (base64-encoded 32 bytes).
    pub fn from_env() -> anyhow::Result<Self> {
        let key_b64 = std::env::var("ENCRYPTION_KEY")
            .map_err(|_| anyhow::anyhow!("ENCRYPTION_KEY environment variable not set"))?;
        let key_bytes = BASE64
            .decode(&key_b64)
            .map_err(|e| anyhow::anyhow!("Failed to decode ENCRYPTION_KEY: {}", e))?;
        if key_bytes.len() != 32 {
            anyhow::bail!(
                "ENCRYPTION_KEY must be 32 bytes (got {} bytes)",
                key_bytes.len()
            );
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(Self { key })
    }

    /// Encrypt plaintext. Returns base64(nonce || ciphertext).
    pub fn encrypt(&self, plaintext: &str) -> anyhow::Result<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        let mut nonce_bytes = [0u8; 12];
        rand::fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&combined))
    }

    /// Decrypt base64(nonce || ciphertext) back to plaintext.
    pub fn decrypt(&self, encrypted: &str) -> anyhow::Result<String> {
        let combined = BASE64
            .decode(encrypted)
            .map_err(|e| anyhow::anyhow!("Failed to decode ciphertext: {}", e))?;

        if combined.len() < 12 {
            anyhow::bail!("Ciphertext too short");
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Decrypted text is not valid UTF-8: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_encryptor() -> Encryptor {
        let key = [42u8; 32]; // fixed key for testing
        Encryptor { key }
    }

    fn other_encryptor() -> Encryptor {
        let key = [99u8; 32]; // different key
        Encryptor { key }
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let enc = test_encryptor();
        let plaintext = "my_secret_password";
        let encrypted = enc.encrypt(plaintext).unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_different_ciphertext_each_time() {
        let enc = test_encryptor();
        let a = enc.encrypt("password").unwrap();
        let b = enc.encrypt("password").unwrap();
        assert_ne!(
            a, b,
            "nonce randomization should produce different ciphertexts"
        );
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let enc = test_encryptor();
        let other = other_encryptor();
        let encrypted = enc.encrypt("secret").unwrap();
        assert!(other.decrypt(&encrypted).is_err());
    }

    #[test]
    fn encrypt_decrypt_empty_string() {
        let enc = test_encryptor();
        let encrypted = enc.encrypt("").unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn encrypt_decrypt_unicode() {
        let enc = test_encryptor();
        let plaintext = "日本語パスワード🔑";
        let encrypted = enc.encrypt(plaintext).unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_invalid_base64_fails() {
        let enc = test_encryptor();
        assert!(enc.decrypt("not-valid-base64!!!").is_err());
    }

    #[test]
    fn decrypt_too_short_ciphertext_fails() {
        let enc = test_encryptor();
        // base64 of less than 12 bytes
        let short = BASE64.encode(&[1u8; 5]);
        assert!(enc.decrypt(&short).is_err());
    }
}
