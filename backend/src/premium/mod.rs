use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::AppError;
use crate::AppState;

#[derive(Deserialize, ToSchema)]
pub struct GenerateRequest {
    /// Prompt for AI content generation
    pub prompt: String,
    /// Target platform (threads or instagram)
    pub platform: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct GenerateResponse {
    /// Generated content
    pub content: String,
}

#[derive(Serialize, ToSchema)]
pub struct AnalyticsResponse {
    /// Number of likes
    pub likes: i32,
    /// Number of replies
    pub replies: i32,
    /// Number of reposts
    pub reposts: i32,
    /// Number of impressions
    pub impressions: i32,
}

#[utoipa::path(
    post,
    path = "/api/ai/generate",
    tag = "premium",
    security(("bearer_auth" = [])),
    request_body = GenerateRequest,
    responses(
        (status = 200, description = "Content generated", body = GenerateResponse),
        (status = 401, description = "Unauthorized"),
        (status = 502, description = "Premium API error")
    )
)]
pub async fn generate_content(
    State(state): State<std::sync::Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, AppError> {
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/v1/ai/generate", state.config.premium_api_url))
        .header("Authorization", format!("Bearer {}", state.config.premium_api_key))
        .json(&serde_json::json!({
            "prompt": req.prompt,
            "platform": req.platform
        }))
        .send()
        .await
        .map_err(|e| AppError::PremiumApi(e.to_string()))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::PremiumApi(e.to_string()))?;

    let content = data["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(Json(GenerateResponse { content }))
}

#[utoipa::path(
    get,
    path = "/api/analytics/{account_id}",
    tag = "premium",
    security(("bearer_auth" = [])),
    params(("account_id" = uuid::Uuid, Path, description = "Account ID")),
    responses(
        (status = 200, description = "Analytics data", body = AnalyticsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 502, description = "Premium API error")
    )
)]
pub async fn get_analytics(
    State(state): State<std::sync::Arc<AppState>>,
    axum::extract::Path(account_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<AnalyticsResponse>, AppError> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "{}/v1/analytics/{}",
            state.config.premium_api_url, account_id
        ))
        .header("Authorization", format!("Bearer {}", state.config.premium_api_key))
        .send()
        .await
        .map_err(|e| AppError::PremiumApi(e.to_string()))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::PremiumApi(e.to_string()))?;

    Ok(Json(AnalyticsResponse {
        likes: data["likes"].as_i64().unwrap_or(0) as i32,
        replies: data["replies"].as_i64().unwrap_or(0) as i32,
        reposts: data["reposts"].as_i64().unwrap_or(0) as i32,
        impressions: data["impressions"].as_i64().unwrap_or(0) as i32,
    }))
}
