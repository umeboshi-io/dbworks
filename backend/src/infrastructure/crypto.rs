use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::Aead,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;

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
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
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
