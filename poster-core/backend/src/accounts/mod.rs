use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::AppState;

#[derive(Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub provider: String,
    pub username: String,
}

pub async fn list(
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    let accounts = sqlx::query_as::<_, crate::db::Account>(
        "SELECT * FROM accounts WHERE user_id = $1",
    )
    .bind(Uuid::nil())
    .fetch_all(&state.db)
    .await?;

    let response = accounts
        .into_iter()
        .map(|a| AccountResponse {
            id: a.id,
            provider: a.provider,
            username: a.username,
        })
        .collect();

    Ok(Json(response))
}

#[derive(Serialize)]
pub struct OAuthUrl {
    pub url: String,
}

pub async fn connect(
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<OAuthUrl>, AppError> {
    let url = format!(
        "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&scope=instagram_basic,instagram_content_publish,threads_basic,threads_content_publish&response_type=code",
        state.config.meta_app_id,
        state.config.meta_redirect_uri
    );

    Ok(Json(OAuthUrl { url }))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
}

pub async fn callback(
    State(state): State<std::sync::Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<CallbackParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = reqwest::Client::new();

    let token_resp = client
        .post("https://graph.facebook.com/v19.0/oauth/access_token")
        .form(&[
            ("client_id", &state.config.meta_app_id),
            ("client_secret", &state.config.meta_app_secret),
            ("redirect_uri", &state.config.meta_redirect_uri),
            ("code", &params.code),
        ])
        .send()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| AppError::MetaApi("No access token".to_string()))?;

    let long_lived = client
        .get("https://graph.facebook.com/v19.0/oauth/access_token")
        .query(&[
            ("grant_type", "fb_exchange_token"),
            ("client_id", &state.config.meta_app_id),
            ("client_secret", &state.config.meta_app_secret),
            ("fb_exchange_token", access_token),
        ])
        .send()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let long_lived_data: serde_json::Value = long_lived
        .json()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let long_token = long_lived_data["access_token"]
        .as_str()
        .unwrap_or(access_token);

    let user_resp = client
        .get("https://graph.facebook.com/v19.0/me")
        .query(&[("fields", "id,name"), ("access_token", long_token)])
        .send()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let user_data: serde_json::Value = user_resp
        .json()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let account_id = Uuid::new_v4();
    let provider_user_id = user_data["id"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let username = user_data["name"]
        .as_str()
        .unwrap_or("")
        .to_string();

    sqlx::query(
        "INSERT INTO accounts (id, user_id, provider, provider_user_id, username, access_token) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(account_id)
    .bind(Uuid::nil())
    .bind("instagram")
    .bind(&provider_user_id)
    .bind(&username)
    .bind(long_token)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "status": "connected",
        "username": username
    })))
}
