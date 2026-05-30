use axum::{extract::{Path, State}, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::middleware::AuthUser;
use crate::error::AppError;
use crate::AppState;

#[derive(Deserialize)]
pub struct CreatePostRequest {
    pub account_id: Uuid,
    pub content: String,
    pub media_url: Option<String>,
    pub media_type: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub platform: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePostRequest {
    pub content: Option<String>,
    pub media_url: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub content: String,
    pub media_url: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub status: String,
    pub platform: String,
    pub account_id: Uuid,
}

pub async fn list(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<PostResponse>>, AppError> {
    let posts = sqlx::query_as::<_, crate::db::Post>(
        "SELECT p.* FROM posts p
         JOIN accounts a ON p.account_id = a.id
         WHERE a.user_id = $1
         ORDER BY p.created_at DESC",
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let response = posts
        .into_iter()
        .map(|p| PostResponse {
            id: p.id,
            content: p.content,
            media_url: p.media_url,
            scheduled_at: p.scheduled_at,
            published_at: p.published_at,
            status: p.status,
            platform: p.platform,
            account_id: p.account_id,
        })
        .collect();

    Ok(Json(response))
}

pub async fn create(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreatePostRequest>,
) -> Result<Json<PostResponse>, AppError> {
    // Verify account belongs to user
    let account = sqlx::query_as::<_, crate::db::Account>(
        "SELECT * FROM accounts WHERE id = $1 AND user_id = $2",
    )
    .bind(req.account_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let post_id = Uuid::new_v4();
    let status = if req.scheduled_at.is_some() {
        "scheduled"
    } else {
        "draft"
    };
    let platform = req.platform.unwrap_or_else(|| "threads".to_string());

    sqlx::query(
        "INSERT INTO posts (id, account_id, content, media_url, media_type, scheduled_at, status, platform) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(post_id)
    .bind(req.account_id)
    .bind(&req.content)
    .bind(&req.media_url)
    .bind(&req.media_type)
    .bind(req.scheduled_at)
    .bind(status)
    .bind(&platform)
    .execute(&state.db)
    .await?;

    Ok(Json(PostResponse {
        id: post_id,
        content: req.content,
        media_url: req.media_url,
        scheduled_at: req.scheduled_at,
        published_at: None,
        status: status.to_string(),
        platform,
        account_id: req.account_id,
    }))
}

pub async fn update(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePostRequest>,
) -> Result<Json<PostResponse>, AppError> {
    let post = sqlx::query_as::<_, crate::db::Post>(
        "SELECT p.* FROM posts p
         JOIN accounts a ON p.account_id = a.id
         WHERE p.id = $1 AND a.user_id = $2",
    )
    .bind(id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let content = req.content.unwrap_or(post.content);
    let media_url = req.media_url.or(post.media_url);
    let scheduled_at = req.scheduled_at.or(post.scheduled_at);

    sqlx::query("UPDATE posts SET content = $1, media_url = $2, scheduled_at = $3 WHERE id = $4")
        .bind(&content)
        .bind(&media_url)
        .bind(scheduled_at)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(PostResponse {
        id,
        content,
        media_url,
        scheduled_at,
        published_at: post.published_at,
        status: post.status,
        platform: post.platform,
        account_id: post.account_id,
    }))
}

pub async fn delete(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query(
        "DELETE FROM posts p USING accounts a
         WHERE p.account_id = a.id AND p.id = $1 AND a.user_id = $2",
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

pub async fn publish(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PostResponse>, AppError> {
    let post = sqlx::query_as::<_, crate::db::Post>(
        "SELECT p.* FROM posts p
         JOIN accounts a ON p.account_id = a.id
         WHERE p.id = $1 AND a.user_id = $2",
    )
    .bind(id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let account = sqlx::query_as::<_, crate::db::Account>(
        "SELECT * FROM accounts WHERE id = $1 AND user_id = $2",
    )
    .bind(post.account_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let meta_client = crate::meta::MetaClient::new(&state.config);
    let result = meta_client
        .publish_post(&account, &post)
        .await?;

    sqlx::query(
        "UPDATE posts SET status = 'published', published_at = NOW(), platform_post_id = $1 WHERE id = $2",
    )
    .bind(&result.post_id)
    .bind(id)
    .execute(&state.db)
    .await?;

    Ok(Json(PostResponse {
        id: post.id,
        content: post.content,
        media_url: post.media_url,
        scheduled_at: post.scheduled_at,
        published_at: Some(Utc::now()),
        status: "published".to_string(),
        platform: post.platform,
        account_id: post.account_id,
    }))
}

#[derive(Serialize)]
pub struct CalendarDay {
    pub date: String,
    pub posts: Vec<PostResponse>,
}

pub async fn calendar(
    State(state): State<std::sync::Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Vec<CalendarDay>>, AppError> {
    let posts = sqlx::query_as::<_, crate::db::Post>(
        "SELECT p.* FROM posts p
         JOIN accounts a ON p.account_id = a.id
         WHERE a.user_id = $1 AND p.scheduled_at IS NOT NULL
         ORDER BY p.scheduled_at",
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await?;

    let mut days: std::collections::HashMap<String, Vec<PostResponse>> =
        std::collections::HashMap::new();

    for post in posts {
        let date = post
            .scheduled_at
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        days.entry(date).or_default().push(PostResponse {
            id: post.id,
            content: post.content,
            media_url: post.media_url,
            scheduled_at: post.scheduled_at,
            published_at: post.published_at,
            status: post.status,
            platform: post.platform,
            account_id: post.account_id,
        });
    }

    let result: Vec<CalendarDay> = days
        .into_iter()
        .map(|(date, posts)| CalendarDay { date, posts })
        .collect();

    Ok(Json(result))
}
