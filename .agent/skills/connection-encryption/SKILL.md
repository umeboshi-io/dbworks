---
description: How connection encryption works and how to manage secrets
---

# Connection Encryption

Database connection passwords are encrypted at rest using AES-256-GCM.

## Architecture

```
                  ENCRYPTION_KEY (env var, base64-encoded 32 bytes)
                          │
                          ▼
  password ──► AES-256-GCM encrypt ──► encrypted_password (stored in DB)
                          │
  encrypted_password ──► AES-256-GCM decrypt ──► password (used at runtime)
```

## Key Management

### Phase 1 — Local Development

- Key stored in environment variable `ENCRYPTION_KEY`
- Generate: `openssl rand -base64 32`

### Phase 2 — SaaS (Future)

- Replace with Cloud KMS (GCP Cloud KMS / AWS KMS)
- Envelope encryption: data key encrypted by KMS master key
- The crypto module should be designed with a trait to swap implementations

## Implementation (`crypto.rs`)

The module should expose:

```rust
pub struct Encryptor {
    key: [u8; 32],
}

impl Encryptor {
    pub fn from_env() -> Result<Self>;
    pub fn encrypt(&self, plaintext: &str) -> Result<String>;  // returns base64
    pub fn decrypt(&self, ciphertext: &str) -> Result<String>;
}
```

- Nonce: randomly generated per encryption (12 bytes), prepended to ciphertext
- Output format: `base64(nonce || ciphertext)`

## Security Considerations

- Never log passwords or encryption keys
- `ConnectionInfo` uses `#[serde(skip_serializing)]` on password field
- Encryption key rotation requires re-encrypting all stored passwords
- In SaaS mode, consider per-organization encryption keys
