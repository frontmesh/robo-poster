#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub meta_app_id: String,
    pub meta_app_secret: String,
    pub meta_redirect_uri: String,
    pub premium_api_url: String,
    pub premium_api_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            meta_app_id: std::env::var("META_APP_ID")
                .expect("META_APP_ID must be set"),
            meta_app_secret: std::env::var("META_APP_SECRET")
                .expect("META_APP_SECRET must be set"),
            meta_redirect_uri: std::env::var("META_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/api/accounts/callback".to_string()),
            premium_api_url: std::env::var("PREMIUM_API_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            premium_api_key: std::env::var("PREMIUM_API_KEY")
                .unwrap_or_default(),
        }
    }
}
