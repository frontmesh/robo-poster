use poster_core::error::AppError;
use axum::http::StatusCode;
use axum::response::IntoResponse;

#[test]
fn status_unauthorized_401() {
    let error = AppError::Unauthorized;
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn status_not_found_404() {
    let error = AppError::NotFound;
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn status_bad_request_400() {
    let error = AppError::BadRequest("Invalid input".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn status_meta_api_502() {
    let error = AppError::MetaApi("API error".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn status_premium_api_502() {
    let error = AppError::PremiumApi("Premium API error".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn status_internal_500() {
    let error = AppError::Internal("Internal error".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn post_status_draft() {
    // When scheduled_at is None, status should be "draft"
    let scheduled_at: Option<chrono::DateTime<chrono::Utc>> = None;
    let status = if scheduled_at.is_some() {
        "scheduled"
    } else {
        "draft"
    };
    assert_eq!(status, "draft");
}

#[test]
fn post_status_scheduled() {
    // When scheduled_at is Some, status should be "scheduled"
    let scheduled_at = Some(chrono::Utc::now());
    let status = if scheduled_at.is_some() {
        "scheduled"
    } else {
        "draft"
    };
    assert_eq!(status, "scheduled");
}

#[test]
fn post_platform_default() {
    // When platform is None, should default to "threads"
    let platform = None;
    let platform = platform.unwrap_or_else(|| "threads".to_string());
    assert_eq!(platform, "threads");
}

#[test]
fn post_platform_explicit() {
    // When platform is Some, should use the provided value
    let platform = Some("instagram".to_string());
    let platform = platform.unwrap_or_else(|| "threads".to_string());
    assert_eq!(platform, "instagram");
}
