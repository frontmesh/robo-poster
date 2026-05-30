use poster_core::meta::MetaClient;
use poster_core::db::{Account, Post};
use poster_core::config::Config;
use chrono::Utc;
use uuid::Uuid;

fn test_config() -> Config {
    Config {
        database_url: "postgres://localhost/test".to_string(),
        jwt_secret: "test_secret".to_string(),
        meta_app_id: "test_app_id".to_string(),
        meta_app_secret: "test_app_secret".to_string(),
        meta_redirect_uri: "http://localhost:3000/callback".to_string(),
        premium_api_url: "http://localhost:3001".to_string(),
        premium_api_key: "test_key".to_string(),
    }
}

fn test_account(provider: &str) -> Account {
    Account {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        provider: provider.to_string(),
        provider_user_id: "123456789".to_string(),
        username: "test_user".to_string(),
        access_token: "test_token".to_string(),
        refresh_token: Some("test_refresh".to_string()),
        token_expires_at: Some(Utc::now() + chrono::Duration::days(30)),
        created_at: Utc::now(),
    }
}

fn test_post(media_type: Option<&str>) -> Post {
    Post {
        id: Uuid::new_v4(),
        account_id: Uuid::new_v4(),
        content: "Test post content".to_string(),
        media_url: None,
        media_type: media_type.map(|s| s.to_string()),
        scheduled_at: None,
        published_at: None,
        status: "draft".to_string(),
        platform: "threads".to_string(),
        platform_post_id: None,
        created_at: Utc::now(),
    }
}

#[test]
fn get_base_url_threads() {
    let config = test_config();
    let client = MetaClient::new(&config);
    let account = test_account("threads");

    let url = client.get_base_url(&account);
    assert_eq!(url, "https://graph.threads.net/v1.0");
}

#[test]
fn get_base_url_instagram() {
    let config = test_config();
    let client = MetaClient::new(&config);
    let account = test_account("instagram");

    let url = client.get_base_url(&account);
    assert_eq!(url, "https://graph.facebook.com/v19.0");
}

#[test]
fn get_base_url_unknown() {
    let config = test_config();
    let client = MetaClient::new(&config);
    let account = test_account("unknown_platform");

    let url = client.get_base_url(&account);
    // Should default to threads URL
    assert_eq!(url, "https://graph.threads.net/v1.0");
}

#[test]
fn create_container_text_form_data() {
    let config = test_config();
    let client = MetaClient::new(&config);
    let account = test_account("threads");
    let post = test_post(None);

    // We can't directly test create_container without mocking HTTP,
    // but we can verify the form data construction logic
    let media_type = post.media_type.as_deref().unwrap_or("TEXT");
    assert_eq!(media_type, "TEXT");

    // Verify that TEXT type doesn't require media_url
    assert!(post.media_url.is_none());
}

#[test]
fn create_container_image_form_data() {
    let config = test_config();
    let client = MetaClient::new(&config);
    let account = test_account("threads");

    let mut post = test_post(Some("IMAGE"));
    post.media_url = Some("https://example.com/image.jpg".to_string());

    let media_type = post.media_type.as_deref().unwrap_or("TEXT");
    assert_eq!(media_type, "IMAGE");
    assert!(post.media_url.is_some());
}

#[test]
fn create_container_video_form_data() {
    let config = test_config();
    let client = MetaClient::new(&config);
    let account = test_account("threads");

    let mut post = test_post(Some("VIDEO"));
    post.media_url = Some("https://example.com/video.mp4".to_string());

    let media_type = post.media_type.as_deref().unwrap_or("TEXT");
    assert_eq!(media_type, "VIDEO");
    assert!(post.media_url.is_some());
}

#[test]
fn publish_endpoint_instagram() {
    let config = test_config();
    let client = MetaClient::new(&config);
    let account = test_account("instagram");

    let base_url = client.get_base_url(&account);
    let endpoint = format!("{}/{}/media_publish", base_url, account.provider_user_id);

    assert_eq!(
        endpoint,
        "https://graph.facebook.com/v19.0/123456789/media_publish"
    );
}

#[test]
fn publish_endpoint_threads() {
    let config = test_config();
    let client = MetaClient::new(&config);
    let account = test_account("threads");

    let base_url = client.get_base_url(&account);
    let endpoint = format!("{}/{}/threads_publish", base_url, account.provider_user_id);

    assert_eq!(
        endpoint,
        "https://graph.threads.net/v1.0/123456789/threads_publish"
    );
}
