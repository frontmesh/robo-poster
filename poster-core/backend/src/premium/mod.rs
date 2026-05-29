use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::AppState;

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub platform: Option<String>,
}

#[derive(Serialize)]
pub struct GenerateResponse {
    pub content: String,
}

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

#[derive(Serialize)]
pub struct AnalyticsResponse {
    pub likes: i32,
    pub replies: i32,
    pub reposts: i32,
    pub impressions: i32,
}

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
