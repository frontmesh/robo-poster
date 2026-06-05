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
}

impl AppState {
    pub fn new(db: sqlx::PgPool, config: config::Config) -> Self {
        Self {
            db,
            config,
            oauth_states: Mutex::new(HashMap::new()),
        }
    }
}
