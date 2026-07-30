use assert_cmd::Command;
use mockito::Server;
use std::fs;
use tempfile::TempDir;

const USER: &str = r#"{"id":"user-123","name":"Ada Lovelace","email":"ada@example.test"}"#;

fn command(server: &Server, config_home: &TempDir) -> Command {
    let mut command = Command::cargo_bin("hevy-rs").unwrap();
    command
        .env("HEVY_API_BASE_URL", server.url())
        .env("HEVY_CONFIG_DIR", config_home.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("APPDATA", config_home.path())
        .env("LOCALAPPDATA", config_home.path())
        .env("HOME", config_home.path());
    command
}

#[test]
fn user_get_prefers_explicit_key_and_emits_stable_json() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/user/info")
        .match_header("api-key", "explicit-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(USER)
        .create();

    let output = command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "explicit-key",
            "user",
            "get",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::from_str::<serde_json::Value>(USER).unwrap()
    );
    assert!(String::from_utf8(output.stderr).unwrap().is_empty());
    request.assert();
}

#[test]
fn user_get_uses_environment_key_before_per_user_configuration() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let config_dir = config_home.path().join("hevy");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "api_key = \"config-key\"\n").unwrap();
    let request = server
        .mock("GET", "/v1/user/info")
        .match_header("api-key", "environment-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(USER)
        .create();

    command(&server, &config_home)
        .env("HEVY_API_KEY", "environment-key")
        .args(["user", "get"])
        .assert()
        .success()
        .stdout("ID: user-123\nName: Ada Lovelace\nEmail: ada@example.test\n")
        .stderr("");

    request.assert();
}

#[test]
fn user_get_uses_per_user_configuration_when_no_higher_precedence_key_exists() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let config_dir = config_home.path().join("hevy");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), "api_key = \"config-key\"\n").unwrap();
    let request = server
        .mock("GET", "/v1/user/info")
        .match_header("api-key", "config-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(USER)
        .create();

    command(&server, &config_home)
        .args(["user", "get"])
        .assert()
        .success();

    request.assert();
}

#[test]
fn invalid_invocation_has_a_json_error_and_exit_code_two() {
    let server = Server::new();
    let config_home = TempDir::new().unwrap();

    let output = command(&server, &config_home)
        .args(["--format", "json", "user", "unknown"])
        .assert()
        .code(2)
        .get_output()
        .clone();

    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["code"], "invocation");
    assert!(error["message"].is_string());
    assert!(output.stdout.is_empty());
}

#[test]
fn missing_key_is_an_authentication_error_without_secret_leakage() {
    let server = Server::new();
    let config_home = TempDir::new().unwrap();

    let output = command(&server, &config_home)
        .args(["--format", "json", "user", "get"])
        .assert()
        .code(3)
        .get_output()
        .clone();

    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "{\"code\":\"authentication\",\"message\":\"No API key was provided. Set HEVY_API_KEY, use --api-key, or configure hevy/config.toml.\"}\n"
    );
    assert!(String::from_utf8(output.stdout).unwrap().is_empty());
}

#[test]
fn api_failures_have_a_stable_json_contract() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/user/info")
        .match_header("api-key", "api-key")
        .with_status(404)
        .with_header("x-request-id", "request-123")
        .create();

    let output = command(&server, &config_home)
        .args(["--format", "json", "--api-key", "api-key", "user", "get"])
        .assert()
        .code(4)
        .get_output()
        .clone();

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stderr).unwrap(),
        serde_json::json!({
            "code": "api",
            "message": "The Hevy API request failed.",
            "status": 404,
            "request_id": "request-123"
        })
    );
    request.assert();
}

#[test]
fn transient_read_failures_are_retried_and_report_exhaustion_as_transport() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/user/info")
        .match_header("api-key", "retry-key")
        .with_status(503)
        .expect(4)
        .create();

    let output = command(&server, &config_home)
        .args(["--format", "json", "--api-key", "retry-key", "user", "get"])
        .assert()
        .code(5)
        .get_output()
        .clone();

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stderr).unwrap(),
        serde_json::json!({
            "code": "transport",
            "message": "The Hevy API remained temporarily unavailable while reading.",
            "status": 503
        })
    );
    request.assert();
}

#[test]
fn unauthorized_response_is_categorized_without_echoing_the_key() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/user/info")
        .match_header("api-key", "very-secret-key")
        .with_status(401)
        .with_body("very-secret-key")
        .create();

    let output = command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "very-secret-key",
            "user",
            "get",
        ])
        .assert()
        .code(3)
        .get_output()
        .clone();

    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "{\"code\":\"authentication\",\"message\":\"The Hevy API rejected the supplied API key.\",\"status\":401}\n"
    );
    assert!(
        !String::from_utf8(output.stdout)
            .unwrap()
            .contains("very-secret-key")
    );
    request.assert();
}
