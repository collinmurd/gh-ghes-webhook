use std::collections::HashMap;

use reqwest::header::HeaderName;

use crate::gh::WebhookDeliveryRequest;

pub trait Forwarder {
    fn forward(&self, payload: WebhookDeliveryRequest);
}

pub struct LocalForwarder {
    url: String,
    client: reqwest::blocking::Client
}

impl LocalForwarder {
    pub fn new(url: String) -> Self {
        let mut clean_url = url.trim().to_string();
        if !clean_url.starts_with("http://") {
            clean_url = format!("http://{}", clean_url);
        }
        LocalForwarder {
            url: clean_url,
            client: reqwest::blocking::Client::builder()
                .user_agent(env!("CARGO_PKG_NAME"))
                .build().unwrap()
        }
    }
}

impl Forwarder for LocalForwarder {
    fn forward(&self, payload: WebhookDeliveryRequest) {
        self.client.post(&self.url)
            .json(&payload.payload)
            .headers(build_headers(payload.headers))
            .send()
            .unwrap();
    }
}

pub struct StdOutForwarder;

impl StdOutForwarder {
    pub fn new() -> Self {
        StdOutForwarder
    }
}

impl Forwarder for StdOutForwarder {
    fn forward(&self, payload: WebhookDeliveryRequest) {
        println!("Received webhook delivery: {:?}", payload);
    }
}

fn build_headers(raw_headers: HashMap<String, String>) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    for (k, v) in raw_headers {
        headers.insert(
            HeaderName::from_bytes(k.as_bytes()).unwrap(), 
            v.parse().unwrap()
        );
    }
    headers
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::build_headers;


    #[test]
    fn test_build_headers() {
        // create a hashmap with some headers using serde_json
        let headers= serde_json::json!({
            "Content-Type": "application/json",
            "X-My-Header": "my-value"
        });
        let headers: HashMap<String, String> = serde_json::from_value(headers).unwrap();
        let result = build_headers(headers);

        assert_eq!(result.len(), 2);
        assert!(result.get("content-type").is_some_and(|v| v == "application/json"));
        assert!(result.get("x-my-header").is_some_and(|v| v == "my-value"));
    }
}