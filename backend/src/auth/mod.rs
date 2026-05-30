use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use crate::AppState;

pub mod middleware;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
}

pub async fn register(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if req.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    if !req.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email".to_string()));
    }

    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "Email already registered".to_string(),
        ));
    }

    let password_hash = hash_password(&req.password);
    let user_id = Uuid::new_v4();

    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(&req.email)
        .bind(&password_hash)
        .execute(&state.db)
        .await?;

    let token = create_jwt(user_id, &state.config.jwt_secret);

    Ok(Json(AuthResponse {
        token,
        user_id,
    }))
}

pub async fn login(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let row = sqlx::query("SELECT id, password_hash FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await?;

    let row = row.ok_or(AppError::Unauthorized)?;
    let user_id: Uuid = row.get("id");
    let password_hash: String = row.get("password_hash");

    if !verify_password(&req.password, &password_hash) {
        return Err(AppError::Unauthorized);
    }

    let token = create_jwt(user_id, &state.config.jwt_secret);

    Ok(Json(AuthResponse {
        token,
        user_id,
    }))
}

fn hash_password(password: &str) -> String {
    use base64::Engine;
    let hash = blake3::hash(password.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(hash.as_bytes())
}

fn verify_password(password: &str, hash: &str) -> bool {
    use base64::Engine;
    let computed = blake3::hash(password.as_bytes());
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(hash)
        .unwrap_or_default();
    computed.as_bytes() == decoded.as_slice()
}

fn create_jwt(user_id: Uuid, secret: &str) -> String {
    use base64::Engine;
    let header = r#"{"alg":"HS256","typ":"JWT"}"#;
    let exp = chrono::Utc::now() + chrono::Duration::hours(24);
    let payload = format!(
        r#"{{"sub":"{}","exp":{}}}"#,
        user_id,
        exp.timestamp()
    );

    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header);
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload);
    let signature = format!("{}.{}", header_b64, payload_b64);

    signature
}
