use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl AuthUser {
    pub fn from_token(token: &str, secret: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 2 {
            return Err(AppError::Unauthorized);
        }

        use base64::Engine;
        let payload_b64 = parts[1];
        let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload_b64)
            .map_err(|_| AppError::Unauthorized)?;

        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
            .map_err(|_| AppError::Unauthorized)?;

        let exp = payload["exp"]
            .as_i64()
            .ok_or(AppError::Unauthorized)?;

        let now = chrono::Utc::now().timestamp();
        if exp < now {
            return Err(AppError::Unauthorized);
        }

        let user_id_str = payload["sub"]
            .as_str()
            .ok_or(AppError::Unauthorized)?;

        let user_id = Uuid::parse_str(user_id_str)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser { user_id })
    }
}

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret".to_string());

    let auth_user = AuthUser::from_token(token, &secret)?;

    req.extensions_mut().insert(auth_user);

    Ok(next.run(req).await)
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AppError::Unauthorized)
    }
}
