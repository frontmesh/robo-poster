pub mod accounts;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod meta;
pub mod posts;
pub mod premium;
pub mod scheduler;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: config::Config,
}
