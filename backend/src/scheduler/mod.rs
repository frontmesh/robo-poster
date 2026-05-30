use chrono::{DateTime, Utc, Duration};
use sqlx::PgPool;

pub async fn run_scheduler(pool: PgPool) {
    tracing::info!("Scheduler started");

    let mut publish_interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    let mut token_interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));

    loop {
        tokio::select! {
            _ = publish_interval.tick() => {
                if let Err(e) = publish_scheduled_posts(&pool).await {
                    tracing::error!("Publish scheduler error: {}", e);
                }
            }
            _ = token_interval.tick() => {
                if let Err(e) = refresh_expiring_tokens(&pool).await {
                    tracing::error!("Token refresh error: {}", e);
                }
            }
        }
    }
}

async fn publish_scheduled_posts(pool: &PgPool) -> Result<(), sqlx::Error> {
    let posts = sqlx::query_as::<_, crate::db::Post>(
        "SELECT p.* FROM posts p
         JOIN accounts a ON p.account_id = a.id
         WHERE p.status = 'scheduled'
         AND p.scheduled_at <= NOW()
         ORDER BY p.scheduled_at ASC
         LIMIT 10",
    )
    .fetch_all(pool)
    .await?;

    if posts.is_empty() {
        return Ok(());
    }

    tracing::info!("Found {} scheduled posts to publish", posts.len());

    for post in posts {
        let account = sqlx::query_as::<_, crate::db::Account>(
            "SELECT * FROM accounts WHERE id = $1",
        )
        .bind(post.account_id)
        .fetch_optional(pool)
        .await?;

        match account {
            Some(account) => {
                // Check if token is expired
                if let Some(expires_at) = account.token_expires_at {
                    if expires_at < Utc::now() {
                        tracing::warn!(
                            "Token expired for account {}, skipping post {}",
                            account.id,
                            post.id
                        );
                        // Mark post as failed
                        sqlx::query(
                            "UPDATE posts SET status = 'failed' WHERE id = $1",
                        )
                        .bind(post.id)
                        .execute(pool)
                        .await?;
                        continue;
                    }
                }

                let config = crate::config::Config::from_env();
                let meta_client = crate::meta::MetaClient::new(&config);

                match meta_client.publish_post(&account, &post).await {
                    Ok(result) => {
                        sqlx::query(
                            "UPDATE posts SET status = 'published', published_at = NOW(), platform_post_id = $1 WHERE id = $2",
                        )
                        .bind(&result.post_id)
                        .bind(post.id)
                        .execute(pool)
                        .await?;
                        tracing::info!("Published post {} to {}", post.id, account.provider);
                    }
                    Err(e) => {
                        tracing::error!("Failed to publish post {}: {}", post.id, e);
                        // Don't mark as failed immediately - might be a transient error
                        // The scheduler will retry on the next tick
                    }
                }
            }
            None => {
                tracing::warn!(
                    "Account {} not found for post {}, marking post as failed",
                    post.account_id,
                    post.id
                );
                sqlx::query(
                    "UPDATE posts SET status = 'failed' WHERE id = $1",
                )
                .bind(post.id)
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}

async fn refresh_expiring_tokens(pool: &PgPool) -> Result<(), sqlx::Error> {
    let accounts = sqlx::query_as::<_, crate::db::Account>(
        "SELECT * FROM accounts
         WHERE token_expires_at IS NOT NULL
         AND token_expires_at < NOW() + INTERVAL '7 days'
         AND refresh_token IS NOT NULL",
    )
    .fetch_all(pool)
    .await?;

    if accounts.is_empty() {
        return Ok(());
    }

    tracing::info!("Checking {} accounts for token refresh", accounts.len());

    let config = crate::config::Config::from_env();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| sqlx::Error::Configuration(e.into()))?;

    for account in accounts {
        if let Some(refresh_token) = &account.refresh_token {
            tracing::info!("Refreshing token for account {} ({})", account.id, account.username);

            let resp = client
                .get("https://graph.facebook.com/v19.0/oauth/access_token")
                .query(&[
                    ("grant_type", "fb_exchange_token"),
                    ("client_id", &config.meta_app_id),
                    ("client_secret", &config.meta_app_secret),
                    ("fb_exchange_token", refresh_token),
                ])
                .send()
                .await;

            match resp {
                Ok(resp) => {
                    let status = resp.status();
                    let body: Result<serde_json::Value, _> = resp.json().await;

                    match body {
                        Ok(data) => {
                            if let Some(error) = data.get("error") {
                                let msg = error["message"].as_str().unwrap_or("Unknown error");
                                tracing::error!(
                                    "Token refresh failed for account {}: {}",
                                    account.id,
                                    msg
                                );
                                continue;
                            }

                            if let Some(new_token) = data["access_token"].as_str() {
                                // Calculate new expiry (60 days from now)
                                let new_expires_at = Utc::now() + Duration::days(59);

                                sqlx::query(
                                    "UPDATE accounts SET access_token = $1, token_expires_at = $2 WHERE id = $3",
                                )
                                .bind(new_token)
                                .bind(new_expires_at)
                                .bind(account.id)
                                .execute(pool)
                                .await?;

                                tracing::info!(
                                    "Refreshed token for account {} (expires: {})",
                                    account.id,
                                    new_expires_at.format("%Y-%m-%d")
                                );
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to parse token refresh response for account {}: {}",
                                account.id,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Token refresh request failed for account {}: {}",
                        account.id,
                        e
                    );
                }
            }
        }
    }

    Ok(())
}
