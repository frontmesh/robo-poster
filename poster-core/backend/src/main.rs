use axum::{middleware, routing::get, routing::post, routing::put, Router};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

mod accounts;
mod auth;
mod config;
mod db;
mod error;
mod meta;
mod posts;
mod premium;
mod scheduler;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: config::Config,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let config = config::Config::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = Arc::new(AppState {
        db: pool.clone(),
        config: config.clone(),
    });

    tokio::spawn(scheduler::run_scheduler(pool.clone()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public_routes = Router::new()
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login));

    let protected_routes = Router::new()
        .route("/api/accounts", get(accounts::list))
        .route("/api/accounts/connect", post(accounts::connect))
        .route("/api/accounts/callback", get(accounts::callback))
        .route("/api/posts", get(posts::list).post(posts::create))
        .route(
            "/api/posts/{id}",
            put(posts::update).delete(posts::delete),
        )
        .route("/api/posts/{id}/publish", post(posts::publish))
        .route("/api/calendar", get(posts::calendar))
        .route("/api/ai/generate", post(premium::generate_content))
        .route(
            "/api/analytics/{account_id}",
            get(premium::get_analytics),
        )
        .layer(middleware::from_fn(auth::middleware::auth_middleware));

    let app = public_routes
        .merge(protected_routes)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("Server running on port 3000");
    axum::serve(listener, app).await.unwrap();
}
