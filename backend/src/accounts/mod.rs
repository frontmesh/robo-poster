use axum::{extract::{State, Path}, Json};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct AccountResponse {
    /// Account ID
    pub id: Uuid,
    /// Provider (instagram or threads)
    pub provider: String,
    /// Provider user ID
    pub provider_user_id: String,
    /// Username on the platform
    pub username: String,
    /// Token expiry time
    pub token_expires_at: Option<DateTime<Utc>>,
    /// Account creation time
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct OAuthUrl {
    /// OAuth authorization URL
    pub url: String,
}

#[utoipa::path(
    get,
    path = "/api/accounts",
    tag = "accounts",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of connected accounts", body = Vec<AccountResponse>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    let accounts = sqlx::query_as::<_, crate::db::Account>(
        "SELECT * FROM accounts WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let response = accounts
        .into_iter()
        .map(|a| AccountResponse {
            id: a.id,
            provider: a.provider,
            provider_user_id: a.provider_user_id,
            username: a.username,
            token_expires_at: a.token_expires_at,
            created_at: a.created_at,
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/accounts/connect",
    tag = "accounts",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "OAuth URL for account connection", body = OAuthUrl),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn connect(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<OAuthUrl>, AppError> {
    let scopes = "instagram_basic,instagram_content_publish,instagram_manage_insights,threads_basic,threads_content_publish,threads_manage_insights";

    let state_token = Uuid::new_v4().to_string();

    state.oauth_states.lock()
        .map_err(|_| AppError::Internal("Failed to lock oauth states".to_string()))?
        .insert(state_token.clone(), auth.user_id);

    let url = format!(
        "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&scope={}&response_type=code&state={}",
        state.config.meta_app_id,
        state.config.meta_redirect_uri,
        scopes,
        state_token
    );

    Ok(Json(OAuthUrl { url }))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/accounts/callback",
    tag = "accounts",
    params(("code" = String, Query, description = "OAuth authorization code")),
    responses(
        (status = 200, description = "Account connected"),
        (status = 400, description = "Invalid code or state"),
        (status = 502, description = "Meta API error")
    )
)]
pub async fn callback(
    State(state): State<std::sync::Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<CallbackParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let state_token = params.state
        .ok_or_else(|| AppError::BadRequest("Missing state parameter".to_string()))?;

    let user_id = state.oauth_states.lock()
        .map_err(|_| AppError::Internal("Failed to lock oauth states".to_string()))?
        .remove(&state_token)
        .ok_or_else(|| AppError::BadRequest("Invalid or expired state parameter".to_string()))?;

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

    if let Some(error) = token_data.get("error") {
        return Err(AppError::MetaApi(
            error["message"].as_str().unwrap_or("Unknown error").to_string(),
        ));
    }

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| AppError::MetaApi("No access token in response".to_string()))?;

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
        .query(&[
            ("fields", "id,name,email"),
            ("access_token", long_token),
        ])
        .send()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let user_data: serde_json::Value = user_resp
        .json()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let fb_user_id = user_data["id"]
        .as_str()
        .ok_or_else(|| AppError::MetaApi("No user ID".to_string()))?;

    let ig_resp = client
        .get(format!("https://graph.facebook.com/v19.0/{}/instagram_business_accounts", fb_user_id))
        .query(&[("access_token", long_token)])
        .send()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let ig_data: serde_json::Value = ig_resp
        .json()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let ig_accounts = ig_data["data"]
        .as_array()
        .ok_or_else(|| AppError::MetaApi("No Instagram accounts found".to_string()))?;

    if ig_accounts.is_empty() {
        return Err(AppError::MetaApi(
            "No Instagram Business accounts found".to_string(),
        ));
    }

    let threads_resp = client
        .get(format!("https://graph.facebook.com/v19.0/{}/threads_profiles", fb_user_id))
        .query(&[("access_token", long_token)])
        .send()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let threads_data: serde_json::Value = threads_resp
        .json()
        .await
        .map_err(|e| AppError::MetaApi(e.to_string()))?;

    let mut connected_accounts = Vec::new();

    for ig_account in ig_accounts {
        let ig_id = ig_account["id"].as_str().unwrap_or("");
        let ig_username = ig_account["username"].as_str().unwrap_or("");

        if ig_id.is_empty() {
            continue;
        }

        let existing = sqlx::query_as::<_, crate::db::Account>(
            "SELECT * FROM accounts WHERE user_id = $1 AND provider_user_id = $2",
        )
        .bind(user_id)
        .bind(ig_id)
        .fetch_optional(&state.db)
        .await?;

        if let Some(existing) = existing {
            sqlx::query(
                "UPDATE accounts SET access_token = $1, token_expires_at = $2 WHERE id = $3",
            )
            .bind(long_token)
            .bind(Utc::now() + Duration::days(59))
            .bind(existing.id)
            .execute(&state.db)
            .await?;

            connected_accounts.push(serde_json::json!({
                "id": existing.id,
                "provider": "instagram",
                "username": ig_username,
                "status": "updated"
            }));
        } else {
            let account_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO accounts (id, user_id, provider, provider_user_id, username, access_token, refresh_token, token_expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            )
            .bind(account_id)
            .bind(user_id)
            .bind("instagram")
            .bind(ig_id)
            .bind(ig_username)
            .bind(long_token)
            .bind(long_token)
            .bind(Utc::now() + Duration::days(59))
            .execute(&state.db)
            .await?;

            connected_accounts.push(serde_json::json!({
                "id": account_id,
                "provider": "instagram",
                "username": ig_username,
                "status": "connected"
            }));
        }
    }

    if let Some(threads_array) = threads_data["data"].as_array() {
        for threads_account in threads_array {
            let threads_id = threads_account["id"].as_str().unwrap_or("");
            let threads_username = threads_account["username"].as_str().unwrap_or("");

            if threads_id.is_empty() {
                continue;
            }

            let existing = sqlx::query_as::<_, crate::db::Account>(
                "SELECT * FROM accounts WHERE user_id = $1 AND provider_user_id = $2",
            )
            .bind(user_id)
            .bind(threads_id)
            .fetch_optional(&state.db)
            .await?;

            if let Some(existing) = existing {
                sqlx::query(
                    "UPDATE accounts SET access_token = $1, token_expires_at = $2 WHERE id = $3",
                )
                .bind(long_token)
                .bind(Utc::now() + Duration::days(59))
                .bind(existing.id)
                .execute(&state.db)
                .await?;

                connected_accounts.push(serde_json::json!({
                    "id": existing.id,
                    "provider": "threads",
                    "username": threads_username,
                    "status": "updated"
                }));
            } else {
                let account_id = Uuid::new_v4();
                sqlx::query(
                    "INSERT INTO accounts (id, user_id, provider, provider_user_id, username, access_token, refresh_token, token_expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(account_id)
                .bind(user_id)
                .bind("threads")
                .bind(threads_id)
                .bind(threads_username)
                .bind(long_token)
                .bind(long_token)
                .bind(Utc::now() + Duration::days(59))
                .execute(&state.db)
                .await?;

                connected_accounts.push(serde_json::json!({
                    "id": account_id,
                    "provider": "threads",
                    "username": threads_username,
                    "status": "connected"
                }));
            }
        }
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "accounts": connected_accounts
    })))
}

#[utoipa::path(
    delete,
    path = "/api/accounts/{id}",
    tag = "accounts",
    security(("bearer_auth" = [])),
    params(("id" = Uuid, Path, description = "Account ID")),
    responses(
        (status = 200, description = "Account deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Account not found")
    )
)]
pub async fn delete(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query(
        "DELETE FROM accounts WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(auth.user_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}
