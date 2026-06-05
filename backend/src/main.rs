use axum::{middleware, routing::get, routing::post, Json, Router};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

use poster_core::accounts;
use poster_core::auth;
use poster_core::config;
use poster_core::posts;
use poster_core::premium;
use poster_core::scheduler;
use poster_core::AppState;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Poster API",
        version = "1.0.0",
        description = "Marketing automation API for Threads & Instagram"
    ),
    paths(
        auth::register,
        auth::login,
        accounts::list,
        accounts::connect,
        accounts::callback,
        accounts::delete,
        posts::list,
        posts::create,
        posts::update,
        posts::delete,
        posts::publish,
        posts::calendar,
        premium::generate_content,
        premium::get_analytics,
    ),
    components(schemas(
        auth::RegisterRequest,
        auth::LoginRequest,
        auth::AuthResponse,
        accounts::AccountResponse,
        accounts::OAuthUrl,
        posts::CreatePostRequest,
        posts::UpdatePostRequest,
        posts::PostResponse,
        posts::CalendarDay,
        premium::GenerateRequest,
        premium::GenerateResponse,
        premium::AnalyticsResponse,
    )),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "accounts", description = "Account management"),
        (name = "posts", description = "Post management"),
        (name = "premium", description = "Premium features (AI, analytics)")
    )
)]
struct ApiDoc;

async fn health(axum::extract::State(state): axum::extract::State<Arc<AppState>>) -> Json<serde_json::Value> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();

    let status = if db_ok { "healthy" } else { "degraded" };

    Json(serde_json::json!({
        "status": status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "database": if db_ok { "connected" } else { "disconnected" }
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .json()
        .init();

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

    let state = Arc::new(AppState::new(pool.clone(), config.clone()));

    tokio::spawn(scheduler::run_scheduler(pool.clone()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .route("/health", get(health))
        .routes(routes!(auth::register, auth::login))
        .routes(routes!(
            accounts::list,
            accounts::connect,
            accounts::callback,
            accounts::delete,
        ))
        .routes(routes!(
            posts::list,
            posts::create,
            posts::update,
            posts::delete,
            posts::publish,
            posts::calendar,
        ))
        .routes(routes!(
            premium::generate_content,
            premium::get_analytics,
        ))
        .layer(middleware::from_fn(auth::middleware::auth_middleware))
        .split_for_parts();

    let app = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("Server running on port 3000");
    tracing::info!("Swagger UI at http://localhost:3000/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}
