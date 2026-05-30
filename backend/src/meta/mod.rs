use crate::config::Config;
use crate::db::{Account, Post};
use serde::{Deserialize, Serialize};

pub struct MetaClient {
    client: reqwest::Client,
    #[allow(dead_code)]
    config: Config,
}

#[derive(Deserialize)]
struct MetaErrorResponse {
    error: MetaError,
}

#[derive(Deserialize)]
struct MetaError {
    message: String,
    #[allow(dead_code)]
    r#type: Option<String>,
    #[allow(dead_code)]
    code: Option<i32>,
}

#[derive(Deserialize)]
struct CreateContainerResponse {
    id: String,
}

#[derive(Deserialize, Debug)]
struct ContainerStatusResponse {
    status: String,
    #[allow(dead_code)]
    id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub post_id: String,
    pub status: String,
}

impl MetaClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            config: config.clone(),
        }
    }

    pub async fn publish_post(
        &self,
        account: &Account,
        post: &Post,
    ) -> Result<PublishResult, crate::error::AppError> {
        let base_url = self.get_base_url(account);

        // Step 1: Create media container
        let container_id = self
            .create_container(&base_url, account, post)
            .await?;

        // Step 2: Wait for container to be ready (especially for media)
        self.wait_for_container(&base_url, account, &container_id)
            .await?;

        // Step 3: Publish the container
        let post_id = self
            .publish_container(&base_url, account, &container_id)
            .await?;

        Ok(PublishResult {
            post_id,
            status: "published".to_string(),
        })
    }

    pub fn get_base_url(&self, account: &Account) -> String {
        match account.provider.as_str() {
            "threads" => "https://graph.threads.net/v1.0".to_string(),
            "instagram" => "https://graph.facebook.com/v19.0".to_string(),
            _ => "https://graph.threads.net/v1.0".to_string(),
        }
    }

    async fn create_container(
        &self,
        base_url: &str,
        account: &Account,
        post: &Post,
    ) -> Result<String, crate::error::AppError> {
        let media_type = post.media_type.as_deref().unwrap_or("TEXT");

        let mut form = vec![
            ("media_type".to_string(), media_type.to_string()),
            ("access_token".to_string(), account.access_token.clone()),
        ];

        // Add text content (required for TEXT type, optional for others)
        if !post.content.is_empty() {
            form.push(("text".to_string(), post.content.clone()));
        }

        // Add media URLs based on type
        if let Some(url) = &post.media_url {
            match media_type {
                "IMAGE" => {
                    form.push(("image_url".to_string(), url.clone()));
                }
                "VIDEO" => {
                    form.push(("video_url".to_string(), url.clone()));
                }
                "CAROUSEL" => {
                    // For carousel, we need to handle differently
                    // For now, just add as image
                    form.push(("image_url".to_string(), url.clone()));
                }
                _ => {}
            }
        }

        // For Instagram, we need to use a different endpoint
        let endpoint = if account.provider == "instagram" {
            format!("{}/{}/media", base_url, account.provider_user_id)
        } else {
            format!("{}/{}", base_url, account.provider_user_id)
        };

        tracing::info!("Creating container with endpoint: {}", endpoint);
        tracing::info!("Form data: {:?}", form);

        let resp = self
            .client
            .post(&endpoint)
            .form(&form)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Meta API request failed: {}", e);
                crate::error::AppError::MetaApi(format!("Request failed: {}", e))
            })?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| {
            crate::error::AppError::MetaApi(format!("Failed to parse response: {}", e))
        })?;

        if !status.is_success() {
            let error_msg = body["error"]["message"]
                .as_str()
                .unwrap_or("Unknown Meta API error");
            tracing::error!("Meta API error ({}): {}", status, error_msg);
            return Err(crate::error::AppError::MetaApi(format!(
                "Meta API error: {}",
                error_msg
            )));
        }

        let container_id = body["id"]
            .as_str()
            .ok_or_else(|| {
                crate::error::AppError::MetaApi("No container ID in response".to_string())
            })?
            .to_string();

        tracing::info!("Container created: {}", container_id);
        Ok(container_id)
    }

    async fn wait_for_container(
        &self,
        base_url: &str,
        account: &Account,
        container_id: &str,
    ) -> Result<(), crate::error::AppError> {
        let max_retries = 10;
        let mut retries = 0;

        loop {
            if retries >= max_retries {
                return Err(crate::error::AppError::MetaApi(
                    "Container processing timed out".to_string(),
                ));
            }

            let url = format!("{}/{}", base_url, container_id);
            let resp = self
                .client
                .get(&url)
                .query(&[
                    ("fields", "status"),
                    ("access_token", &account.access_token),
                ])
                .send()
                .await
                .map_err(|e| crate::error::AppError::MetaApi(e.to_string()))?;

            let body: serde_json::Value = resp.json().await.map_err(|e| {
                crate::error::AppError::MetaApi(format!("Failed to parse status: {}", e))
            })?;

            let status = body["status"].as_str().unwrap_or("UNKNOWN");

            tracing::info!(
                "Container {} status: {} (attempt {}/{})",
                container_id,
                status,
                retries + 1,
                max_retries
            );

            match status {
                "FINISHED" => return Ok(()),
                "ERROR" => {
                    let error_msg = body["error"]["message"]
                        .as_str()
                        .unwrap_or("Container processing failed");
                    return Err(crate::error::AppError::MetaApi(format!(
                        "Container error: {}",
                        error_msg
                    )));
                }
                "EXPIRED" => {
                    return Err(crate::error::AppError::MetaApi(
                        "Container expired".to_string(),
                    ));
                }
                _ => {
                    // Still processing, wait and retry
                    retries += 1;
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
            }
        }
    }

    async fn publish_container(
        &self,
        base_url: &str,
        account: &Account,
        container_id: &str,
    ) -> Result<String, crate::error::AppError> {
        let endpoint = if account.provider == "instagram" {
            format!("{}/{}/media_publish", base_url, account.provider_user_id)
        } else {
            format!("{}/{}/threads_publish", base_url, account.provider_user_id)
        };

        let resp = self
            .client
            .post(&endpoint)
            .form(&[
                ("creation_id", container_id),
                ("access_token", &account.access_token),
            ])
            .send()
            .await
            .map_err(|e| crate::error::AppError::MetaApi(e.to_string()))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await.map_err(|e| {
            crate::error::AppError::MetaApi(format!("Failed to parse publish response: {}", e))
        })?;

        if !status.is_success() {
            let error_msg = body["error"]["message"]
                .as_str()
                .unwrap_or("Failed to publish");
            return Err(crate::error::AppError::MetaApi(format!(
                "Publish error: {}",
                error_msg
            )));
        }

        let post_id = body["id"]
            .as_str()
            .ok_or_else(|| {
                crate::error::AppError::MetaApi("No post ID in publish response".to_string())
            })?
            .to_string();

        tracing::info!("Post published successfully: {}", post_id);
        Ok(post_id)
    }

    pub async fn get_post_insights(
        &self,
        account: &Account,
        post_id: &str,
    ) -> Result<serde_json::Value, crate::error::AppError> {
        let base_url = self.get_base_url(account);
        let url = format!("{}/{}/insights", base_url, post_id);

        let resp = self
            .client
            .get(&url)
            .query(&[
                ("metric", "impressions,reach,likes,comments,shares"),
                ("access_token", &account.access_token),
            ])
            .send()
            .await
            .map_err(|e| crate::error::AppError::MetaApi(e.to_string()))?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            crate::error::AppError::MetaApi(format!("Failed to get insights: {}", e))
        })?;

        Ok(body)
    }
}
