use std::{env, path::Path, process::{Child, Command, Stdio}};

use httpmock::MockServer;
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
    let result = run_cli_forward(vec!["--org", "test"]).unwrap().wait_with_output().unwrap();

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Organization webhooks are not supported."));
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_ctrl_c() {
    let gh_server = MockGhServer::new();
    gh_server.add_all_mocks();
    let host = format!("localhost:{}", gh_server.server.port());

    let child = run_cli_forward(vec!["--github-host", host.as_str(), "--repo", "org/repo"]).unwrap();

    // sleep for a second to allow the CLI to start
    std::thread::sleep(std::time::Duration::from_secs(1));

    // send a SIGINT to the child process (doesn't work that way on windows)
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(child.id() as i32), nix::sys::signal::SIGINT
    ).unwrap();

    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("Deleting CLI webhook"));
}

#[test]
fn test_forward_to_stdout() {
    let gh_server = MockGhServer::new();
    gh_server.add_all_mocks();
    let host = format!("localhost:{}", gh_server.server.port());

    let mut child = run_cli_forward(vec!["--github-host", host.as_str(), "--repo", "org/repo"]).unwrap();

    // sleep for a second to allow the CLI to grab webhook deliveries
    std::thread::sleep(std::time::Duration::from_secs(1));
    child.kill().unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("Forwarding event: 1"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Content-Type: application/json"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("{\"issue\":{\"body\":\"Test Body\",\"title\":\"Test Issue\"}}"));
}

#[test]
fn test_foward_to_local_server() {
    // mock gh server
    let gh_server = MockGhServer::new();
    gh_server.add_all_mocks();
    let host = format!("localhost:{}", gh_server.server.port());

    // mock local server
    let mock_reciever = MockServer::start();
    let mock_reciever_endpoint= create_mock_reciever_endpoint(&mock_reciever);
    let url = format!("localhost:{}/test", mock_reciever.port());

    let mut child = run_cli_forward(vec!["--github-host", host.as_str(), "--repo", "org/repo", "--url", url.as_str()]).unwrap();

    // sleep for a second to allow the CLI to grab webhook deliveries. should forward once
    std::thread::sleep(std::time::Duration::from_secs(7));
    child.kill().unwrap();

    mock_reciever_endpoint.assert();
}


fn run_cli_forward(mut args: Vec<&str>) -> Result<Child, ()> {
    args.insert(0, "forward");

    let child = Command::new("target/debug/gh-ghes-webhook")
        .env("PATH", add_mock_gh_to_path())
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    match child {
        Ok(child) => {
            Ok(child)
        }
        Err(_) => Err(())
    }
}

fn add_mock_gh_to_path() -> String {
    let current_dir = env::current_dir().unwrap();
    let test_path = current_dir.join("tests").join("bin");
    let path = env::var("PATH").unwrap();
    format!("{}:{}", test_path.display(), path)
}

fn payload() -> serde_json::Value {
    json!({
        "issue": {
            "title": "Test Issue",
            "body": "Test Body"
        }
    })
}


fn create_mock_reciever_endpoint(server: &MockServer) -> httpmock::Mock {
    server.mock(|when, then| {
        when.method(httpmock::Method::POST)
            .path("/test")
            .header("Content-Type", "application/json")
            .body(payload().to_string());
        then.status(200);
    })
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
                    "name": "cli",
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
                .body(json!([
                    {
                        "id": 1,
                        "delivered_at": "2099-08-01T00:00:00Z",
                        "event": "issues",
                        "action": "opened"
                    }
                ]).to_string());
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
                        "payload": payload()
                    }
                }).to_string());
        });
    }
}
