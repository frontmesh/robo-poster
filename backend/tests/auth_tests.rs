use poster_core::auth::{hash_password, verify_password, create_jwt};
use poster_core::auth::middleware::AuthUser;
use uuid::Uuid;

#[test]
fn hash_password_deterministic() {
    let pw = "test_password_123";
    let hash1 = hash_password(pw);
    let hash2 = hash_password(pw);
    assert_eq!(hash1, hash2);
}

#[test]
fn hash_password_unique() {
    let hash1 = hash_password("password1");
    let hash2 = hash_password("password2");
    assert_ne!(hash1, hash2);
}

#[test]
fn verify_password_correct() {
    let pw = "my_secure_password";
    let hash = hash_password(pw);
    assert!(verify_password(pw, &hash));
}

#[test]
fn verify_password_wrong() {
    let hash = hash_password("correct_password");
    assert!(!verify_password("wrong_password", &hash));
}

#[test]
fn verify_password_tampered() {
    // Valid base64 but wrong content
    let tampered = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        b"not_a_real_hash",
    );
    assert!(!verify_password("any_password", &tampered));
}

#[test]
fn create_jwt_structure() {
    let user_id = Uuid::new_v4();
    let secret = "test_secret_key";
    let token = create_jwt(user_id, secret);

    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts: header.payload.signature");

    // Each part should be valid base64url
    use base64::Engine;
    assert!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[0]).is_ok());
    assert!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]).is_ok());
    assert!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[2]).is_ok());
}

#[test]
fn from_token_valid() {
    let user_id = Uuid::new_v4();
    let secret = "test_secret";
    let token = create_jwt(user_id, secret);

    let auth_user = AuthUser::from_token(&token, secret).unwrap();
    assert_eq!(auth_user.user_id, user_id);
}

#[test]
fn from_token_expired() {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let user_id = Uuid::new_v4();
    let secret = "test_secret";

    // Create a token with expired timestamp
    let header = r#"{"alg":"HS256","typ":"JWT"}"#;
    let exp = chrono::Utc::now() - chrono::Duration::hours(1); // 1 hour ago
    let payload = format!(r#"{{"sub":"{}","exp":{}}}"#, user_id, exp.timestamp());

    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header);
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&payload);

    let signing_input = format!("{}.{}", header_b64, payload_b64);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signing_input.as_bytes());
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&mac.finalize().into_bytes());

    let token = format!("{}.{}", signing_input, signature);

    let result = AuthUser::from_token(&token, secret);
    assert!(result.is_err());
}

#[test]
fn from_token_no_dots() {
    let result = AuthUser::from_token("not-a-jwt-token", "secret");
    assert!(result.is_err());
}

#[test]
fn from_token_too_many_parts() {
    let result = AuthUser::from_token("a.b.c.d", "secret");
    assert!(result.is_err());
}

#[test]
fn from_token_bad_payload() {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let secret = "test_secret";

    // Create a token with invalid JSON payload
    let header = r#"{"alg":"HS256","typ":"JWT"}"#;
    let payload = "not-valid-json";

    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header);
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload);

    let signing_input = format!("{}.{}", header_b64, payload_b64);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signing_input.as_bytes());
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&mac.finalize().into_bytes());

    let token = format!("{}.{}", signing_input, signature);

    let result = AuthUser::from_token(&token, secret);
    assert!(result.is_err());
}

#[test]
fn from_token_missing_exp() {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let user_id = Uuid::new_v4();
    let secret = "test_secret";

    // Create a token without exp field
    let header = r#"{"alg":"HS256","typ":"JWT"}"#;
    let payload = format!(r#"{{"sub":"{}"}}"#, user_id);

    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header);
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&payload);

    let signing_input = format!("{}.{}", header_b64, payload_b64);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signing_input.as_bytes());
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&mac.finalize().into_bytes());

    let token = format!("{}.{}", signing_input, signature);

    let result = AuthUser::from_token(&token, secret);
    assert!(result.is_err());
}

#[test]
fn from_token_wrong_secret() {
    let user_id = Uuid::new_v4();
    let token = create_jwt(user_id, "correct_secret");

    let result = AuthUser::from_token(&token, "wrong_secret");
    assert!(result.is_err());
}

#[test]
fn from_token_missing_sub() {
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let secret = "test_secret";

    // Create a token without sub field
    let header = r#"{"alg":"HS256","typ":"JWT"}"#;
    let exp = chrono::Utc::now() + chrono::Duration::hours(24);
    let payload = format!(r#"{{"exp":{}}}"#, exp.timestamp());

    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header);
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&payload);

    let signing_input = format!("{}.{}", header_b64, payload_b64);

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signing_input.as_bytes());
    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&mac.finalize().into_bytes());

    let token = format!("{}.{}", signing_input, signature);

    let result = AuthUser::from_token(&token, secret);
    assert!(result.is_err());
}
