pub async fn run_scheduler(pool: sqlx::PgPool) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        if let Err(e) = publish_scheduled_posts(&pool).await {
            tracing::error!("Scheduler error: {}", e);
        }

        if let Err(e) = refresh_expiring_tokens(&pool).await {
            tracing::error!("Token refresh error: {}", e);
        }
    }
}

async fn publish_scheduled_posts(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let posts = sqlx::query_as::<_, crate::db::Post>(
        "SELECT * FROM posts WHERE status = 'scheduled' AND scheduled_at <= NOW()",
    )
    .fetch_all(pool)
    .await?;

    for post in posts {
        let account = sqlx::query_as::<_, crate::db::Account>(
            "SELECT * FROM accounts WHERE id = $1",
        )
        .bind(post.account_id)
        .fetch_optional(pool)
        .await?;

        if let Some(account) = account {
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
                    tracing::info!("Published post {}", post.id);
                }
                Err(e) => {
                    tracing::error!("Failed to publish post {}: {}", post.id, e);
                }
            }
        }
    }

    Ok(())
}

async fn refresh_expiring_tokens(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let accounts = sqlx::query_as::<_, crate::db::Account>(
        "SELECT * FROM accounts WHERE token_expires_at IS NOT NULL AND token_expires_at < NOW() + INTERVAL '7 days'",
    )
    .fetch_all(pool)
    .await?;

    let config = crate::config::Config::from_env();
    let client = reqwest::Client::new();

    for account in accounts {
        if let Some(refresh_token) = &account.refresh_token {
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

            if let Ok(resp) = resp {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(new_token) = data["access_token"].as_str() {
                        sqlx::query("UPDATE accounts SET access_token = $1 WHERE id = $2")
                            .bind(new_token)
                            .bind(account.id)
                            .execute(pool)
                            .await?;
                        tracing::info!("Refreshed token for account {}", account.id);
                    }
                }
            }
        }
    }

    Ok(())
}
