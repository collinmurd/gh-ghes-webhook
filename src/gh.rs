use std::process::Command;
use serde_json::json;

pub struct GitHub {
    host: String,
    url: String,
    client: reqwest::Client
}

impl GitHub {
    pub fn new_with_repo(host: String, repo: String) -> Self {
        let url = format!("https://api.{}/repos/{}/hooks", host, repo);

        GitHub {
            host: host,
            url: url,
            client: reqwest::Client::builder()
                .user_agent(env!("CARGO_PKG_NAME"))
                .build().unwrap()
        }
    }

    pub fn new_with_org(host: String, org: String) -> Self {
        let url = format!("https://api.{}/orgs/{}/hooks", host, org);

        GitHub {
            host: host,
            url: url,
            client: reqwest::Client::builder()
                .user_agent(env!("CARGO_PKG_NAME"))
                .build().unwrap()
        }
    }

    pub async fn create_webhook(&self) -> anyhow::Result<CreateWebhookResponse> {
        let token = self.get_auth_token().unwrap();
        let body = json!({
            "name": "cli",
            "active": true,
            "events": ["*"],
            "config": {
                "content_type": "json"
            }
        });

        let req = self.client.post(&self.url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?
            .json::<CreateWebhookResponse>()
            .await?;

        Ok(req)
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


#[derive(serde::Deserialize, Debug)]
pub struct CreateWebhookResponse {
    id: u32,
    name: String,
    active: bool,
    events: Vec<String>,
}