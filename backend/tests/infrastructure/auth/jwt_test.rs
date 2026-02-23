use dbworks_backend::domain::user::AppUser;
use dbworks_backend::infrastructure::auth::jwt::Claims;

/// Smoke test: verifies that the CryptoProvider required by jsonwebtoken is
/// properly configured.  If the `rust_crypto` feature flag is removed from
/// jsonwebtoken in Cargo.toml, this test will panic with:
///   "Could not automatically determine the process-level CryptoProvider"
#[test]
fn jwt_roundtrip_smoke_test() {
    let user = AppUser {
        id: uuid::Uuid::new_v4(),
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        auth_provider: Some("google".to_string()),
        provider_id: Some("12345".to_string()),
        role: "member".to_string(),
        avatar_url: None,
        created_at: None,
        updated_at: None,
    };

    let secret = "test-secret-key";

    // This will panic if CryptoProvider is not available
    let token = Claims::generate_token(&user, secret)
        .expect("JWT encoding must succeed — is CryptoProvider configured?");

    // Verify decode also works
    let claims = Claims::decode(&token, secret)
        .expect("JWT decoding must succeed — is CryptoProvider configured?");

    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.sub, user.id.to_string());
}
