use axum::{extract::{State, Path}, Json};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::AppState;

#[derive(Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub username: String,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

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

#[derive(Serialize)]
pub struct OAuthUrl {
    pub url: String,
}

pub async fn connect(
    State(state): State<std::sync::Arc<AppState>>,
) -> Result<Json<OAuthUrl>, AppError> {
    let scopes = "instagram_basic,instagram_content_publish,instagram_manage_insights,threads_basic,threads_content_publish,threads_manage_insights";
    let url = format!(
        "https://www.facebook.com/v19.0/dialog/oauth?client_id={}&redirect_uri={}&scope={}&response_type=code&state=poster",
        state.config.meta_app_id,
        state.config.meta_redirect_uri,
        scopes
    );

    Ok(Json(OAuthUrl { url }))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
}

pub async fn callback(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
    axum::extract::Query(params): axum::extract::Query<CallbackParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = reqwest::Client::new();

    // Step 1: Exchange code for short-lived token
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

    // Step 2: Exchange for long-lived token (60 days)
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

    // Step 3: Get user info from Facebook
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

    // Step 4: Get Instagram business accounts connected to this Facebook user
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
        .ok_or_else(|| AppError::MetaApi("No Instagram accounts found. Make sure you have an Instagram Business account connected to your Facebook page.".to_string()))?;

    if ig_accounts.is_empty() {
        return Err(AppError::MetaApi(
            "No Instagram Business accounts found. Please connect an Instagram Business account to your Facebook page first.".to_string(),
        ));
    }

    // Step 5: Get Threads profiles
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

    // Step 6: Store accounts
    let mut connected_accounts = Vec::new();

    // Store Instagram accounts
    for ig_account in ig_accounts {
        let ig_id = ig_account["id"].as_str().unwrap_or("");
        let ig_username = ig_account["username"].as_str().unwrap_or("");

        if ig_id.is_empty() {
            continue;
        }

        // Check if account already exists for this user
        let existing = sqlx::query_as::<_, crate::db::Account>(
            "SELECT * FROM accounts WHERE user_id = $1 AND provider_user_id = $2",
        )
        .bind(auth.user_id)
        .bind(ig_id)
        .fetch_optional(&state.db)
        .await?;

        if let Some(existing) = existing {
            // Update token
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
            .bind(auth.user_id)
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

    // Store Threads accounts
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
            .bind(auth.user_id)
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
                .bind(auth.user_id)
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
