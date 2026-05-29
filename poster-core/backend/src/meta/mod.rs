use crate::config::Config;
use crate::db::{Account, Post};
use serde::{Deserialize, Serialize};

pub struct MetaClient {
    client: reqwest::Client,
    config: Config,
}

#[derive(Serialize)]
struct CreateContainerRequest {
    media_type: String,
    text: Option<String>,
    image_url: Option<String>,
    video_url: Option<String>,
}

#[derive(Deserialize)]
struct CreateContainerResponse {
    id: String,
}

#[derive(Deserialize)]
struct ContainerStatus {
    status: String,
}

impl MetaClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            config: config.clone(),
        }
    }

    pub async fn publish_post(
        &self,
        account: &Account,
        post: &Post,
    ) -> Result<String, crate::error::AppError> {
        let base_url = "https://graph.threads.net/v1.0";

        let media_type = post.media_type.as_deref().unwrap_or("TEXT");

        let mut form = vec![
            ("media_type".to_string(), media_type.to_string()),
            ("access_token".to_string(), account.access_token.clone()),
        ];

        if !post.content.is_empty() {
            form.push(("text".to_string(), post.content.clone()));
        }

        if let Some(url) = &post.media_url {
            match media_type {
                "IMAGE" => form.push(("image_url".to_string(), url.clone())),
                "VIDEO" => form.push(("video_url".to_string(), url.clone())),
                _ => {}
            }
        }

        let create_resp = self
            .client
            .post(format!("{}/{}", base_url, account.provider_user_id))
            .form(&form)
            .send()
            .await
            .map_err(|e| crate::error::AppError::MetaApi(e.to_string()))?;

        let create_data: CreateContainerResponse = create_resp
            .json()
            .await
            .map_err(|e| crate::error::AppError::MetaApi(e.to_string()))?;

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let publish_resp = self
            .client
            .post(format!(
                "{}/{}/threads_publish",
                base_url, account.provider_user_id
            ))
            .form(&[
                ("creation_id", &create_data.id),
                ("access_token", &account.access_token),
            ])
            .send()
            .await
            .map_err(|e| crate::error::AppError::MetaApi(e.to_string()))?;

        let publish_data: serde_json::Value = publish_resp
            .json()
            .await
            .map_err(|e| crate::error::AppError::MetaApi(e.to_string()))?;

        publish_data["id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| {
                crate::error::AppError::MetaApi("No post ID in response".to_string())
            })
    }
}
