use std::process::Command;

use chrono::{DateTime, Utc};
use serde_json::Value;

pub struct GitHub {
    url: String,
    client: reqwest::blocking::Client
}

impl GitHub {
    pub fn new_with_repo(host: String, repo: String) -> Self {
        let url = format!("https://api.{}/repos/{}/hooks", host, repo);

        GitHub {
            url: url,
            client: reqwest::blocking::Client::builder()
                .user_agent(env!("CARGO_PKG_NAME"))
                .build().unwrap()
        }
    }

    pub fn new_with_org(host: String, org: String) -> Self {
        let url = format!("https://api.{}/orgs/{}/hooks", host, org);

        GitHub {
            url: url,
            client: reqwest::blocking::Client::builder()
                .user_agent(env!("CARGO_PKG_NAME"))
                .build().unwrap()
        }
    }

    pub fn create_webhook(&self, secret: Option<String>, events: Vec<String>) -> anyhow::Result<CreateWebhookResponse> {
        let token = self.get_auth_token().unwrap();
        let body = CreateWebhookPayload {
            name: "cli".to_string(),
            active: true,
            events: events,
            config: WebhookConfig {
                content_type: WebhookContentType::Json,
                secret: secret,
            }
        };

        let resp = self.client.post(&self.url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .unwrap() // TODO handle error
            .json::<CreateWebhookResponse>()
            .unwrap(); // TODO handle error

        Ok(resp)
    }

    pub fn get_webhook_deliveries(&self, id: u32) -> anyhow::Result<Vec<WebhookDelivery>> {
        let token = self.get_auth_token().unwrap();
        let url = format!("{}/{}/deliveries?per_page=100", &self.url, id);
        let resp = self.client.get(&url)
            .bearer_auth(token)
            .send()
            .unwrap() // TODO handle error;
            .json::<Vec<WebhookDelivery>>()
            .unwrap(); // TODO handle error

        Ok(resp)
    }

    pub fn get_webhook_delivery_details(&self, webhook_id: u32, delivery_id: u64) -> anyhow::Result<WebhookDeliveryDetails> {
        let token = self.get_auth_token().unwrap();
        let url = format!("{}/{}/deliveries/{}", &self.url, webhook_id, delivery_id);
        let resp = self.client.get(&url)
            .bearer_auth(token)
            .send()
            .unwrap() // TODO handle error
            .json::<WebhookDeliveryDetails>()
            .unwrap(); // TODO handle error

        Ok(resp)
    }

    fn get_auth_token(&self) -> Result<String, String> {
        let output = Command::new("gh")
            .args(["auth", "token"])
            .output();

        match output {
            Ok(res) => return Ok(String::from_utf8(res.stdout).unwrap().trim_ascii().to_string()),
            Err(_) => return Err("Failed to get auth token".into()),
        }
    }
}

#[derive(serde::Serialize, Debug)]
enum WebhookContentType {
    Json,
}
#[derive(serde::Serialize, Debug)]
struct WebhookConfig {
    content_type: WebhookContentType,
    secret: Option<String>,
}
#[derive(serde::Serialize, Debug)]
struct CreateWebhookPayload {
    name: String,
    active: bool,
    events: Vec<String>,
    config: WebhookConfig,
}

#[derive(serde::Deserialize, Debug)]
pub struct CreateWebhookResponse {
    pub id: u32,
    pub name: String,
    pub active: bool,
    pub events: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct WebhookDelivery {
    pub id: u64,
    pub delivered_at: DateTime<Utc>,
    pub event: String,
    pub action: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct WebhookDeliveryRequest {
    pub headers: Value,
    pub payload: Value,
}
#[derive(serde::Deserialize, Debug)]
pub struct WebhookDeliveryDetails {
    pub id: u64,
    pub delivered_at: DateTime<Utc>,
    pub event: String,
    pub action: String,
    pub request: WebhookDeliveryRequest,
}