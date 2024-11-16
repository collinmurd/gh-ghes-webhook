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