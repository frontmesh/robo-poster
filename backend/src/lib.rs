pub mod accounts;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod meta;
pub mod posts;
pub mod premium;
pub mod scheduler;

use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: config::Config,
    pub oauth_states: Mutex<HashMap<String, Uuid>>,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(db: sqlx::PgPool, config: config::Config) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            db,
            config,
            oauth_states: Mutex::new(HashMap::new()),
            http_client,
        }
    }
}
