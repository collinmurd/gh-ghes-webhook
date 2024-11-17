use std::{env, path::Path, process::{Command, Output}};

use httpmock::{MockServer};
use serde_json::json;

#[test]
fn test_run_help() {
    assert!(run_cli_forward(vec!["-h"]).is_ok());
}

#[test]
fn test_mock_gh() {
    // verify the mock gh cli is working
    let result = Command::new("gh")
        .env("PATH", add_mock_gh_to_path())
        .args(["auth", "token"])
        .output();

    assert!(result.is_ok());
    assert!(result.unwrap().stdout == b"gh_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n");
}

#[test]
fn test_run_with_org() {
    let result = run_cli_forward(vec!["--org", "test"]);
    assert!(result.is_err());
    assert!(result.unwrap_err().stderr.contains("Organization webhooks are not supported."));
}

#[test]
fn test_ctrl_c() {
}

struct CliError {
    status: i32,
    spawn_error: Option<std::io::Error>,
    stderr: String,
}

fn run_cli_forward(mut args: Vec<&str>) -> Result<Output, CliError> {
    if !Path::new("target/debug/gh-ghes-webhook").exists() {
        println!("Build the CLI with `cargo build` before running integration tests.");
        return Err(CliError {
            status: 1,
            spawn_error: None,
            stderr: String::from("CLI not found. Please build the CLI before running integration tests: `cargo build`")
        });
    }

    args.insert(0, "forward");

    let result = Command::new("target/debug/gh-ghes-webhook")
        .env("PATH", add_mock_gh_to_path())
        .args(args)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(output)
            } else {
                Err(CliError {
                    status: output.status.code().unwrap(),
                    spawn_error: None,
                    stderr: String::from_utf8(output.stderr).unwrap()
                })
            }
        }
        Err(e) => {
            Err(CliError {
                status: 1,
                spawn_error: Some(e),
                stderr: String::from("Failed to spawn CLI")
            })
        }
    }
}

fn add_mock_gh_to_path() -> String {
    let current_dir = env::current_dir().unwrap();
    let test_path = current_dir.join("tests");
    let path = env::var("PATH").unwrap();
    format!("{}:{}", test_path.display(), path)
}

struct MockGhServer {
    server: MockServer,
}

impl MockGhServer {
    fn new() -> Self {
        MockGhServer {
            server: MockServer::start()
        }
    }

    fn add_all_mocks(&self) {
        self.add_create_webhook();
        self.add_delete_webhook();
        self.add_get_webhook_deliveries();
        self.add_get_webhook_delivery_details();
    }

    fn add_create_webhook(&self) {
        self.server.mock(|when, then| {
            when.method(httpmock::Method::POST)
                .path("/repos/org/repo/hooks");
            then.status(200)
                .body(json!({
                    "id": 1,
                    "config": {
                        "url": "http://localhost:8080",
                        "content_type": "json",
                        "secret": "test"
                    },
                    "events": ["issues"],
                    "active": true
                }).to_string());
        });
    }

    fn add_delete_webhook(&self) {
        self.server.mock(|when, then| {
            when.method(httpmock::Method::DELETE)
                .path("/repos/org/repo/hooks/1");
            then.status(204);
        });
    }

    fn add_get_webhook_deliveries(&self) {
        self.server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/repos/org/repo/hooks/1/deliveries");
            then.status(200)
                .body(json!([]).to_string());
        });
    }

    fn add_get_webhook_delivery_details(&self) {
        self.server.mock(|when, then| {
            when.method(httpmock::Method::GET)
                .path("/repos/org/repo/hooks/1/deliveries/1");
            then.status(200)
                .body(json!({
                    "id": 1,
                    "delivered_at": "2021-08-01T00:00:00Z",
                    "event": "issues",
                    "action": "opened",
                    "request": {
                        "headers": {
                            "Content-Type": "application/json"
                        },
                        "payload": {
                            "issue": {
                                "title": "Test Issue",
                                "body": "Test Body"
                            }
                        }
                    }
                }).to_string());
        });
    }
}
