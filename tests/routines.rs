use assert_cmd::Command;
use mockito::Server;
use std::{
    io::Read,
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

fn command(server: &Server, config_home: &TempDir) -> Command {
    let mut command = Command::cargo_bin("hevy-rs").unwrap();
    command
        .env_remove("HEVY_API_KEY")
        .env("HEVY_API_BASE_URL", server.url())
        .env("HEVY_CONFIG_DIR", config_home.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("APPDATA", config_home.path())
        .env("LOCALAPPDATA", config_home.path())
        .env("HOME", config_home.path());
    command
}

#[test]
fn routines_list_makes_the_documented_paginated_request_and_normalizes_json() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/routines")
        .match_header("api-key", "api-key")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "2".into()),
            mockito::Matcher::UrlEncoded("pageSize".into(), "10".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":2,"page_count":3,"routines":[{"id":"routine-1","title":"Upper"}]}"#)
        .create();

    let output = command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routines",
            "list",
            "--page",
            "2",
            "--page-size",
            "10",
        ])
        .assert()
        .success()
        .get_output()
        .clone();

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::json!({
            "items": [{"id": "routine-1", "title": "Upper"}],
            "page": 2,
            "page_count": 3
        })
    );
    assert!(output.stderr.is_empty());
    request.assert();
}

#[test]
fn routines_list_has_readable_default_output() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/routines")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":1,"routines":[{"id":"routine-1","title":"Upper"}]}"#)
        .create();

    command(&server, &config_home)
        .args(["--api-key", "api-key", "routines", "list"])
        .assert()
        .success()
        .stdout("Page: 1 of 1\n- Upper (routine-1)\n")
        .stderr("");

    request.assert();
}

#[test]
fn routine_get_returns_the_documented_resource() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/routines/routine-123")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"routine":{"id":"routine-123","title":"Upper","folder_id":"folder-1","exercises":[]}}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format", "json", "--api-key", "api-key", "routines", "get", "routine-123",
        ])
        .assert()
        .success()
        .stdout("{\"routine\":{\"exercises\":[],\"folder_id\":\"folder-1\",\"id\":\"routine-123\",\"title\":\"Upper\"}}\n")
        .stderr("");

    request.assert();
}

#[test]
fn routine_mutations_preserve_complete_nested_payloads_and_support_dry_runs() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let payload = r#"{"routine":{"title":"Upper","folder_id":1,"notes":"","exercises":[{"exercise_template_id":"bench","rest_seconds":120,"sets":[{"type":"normal","weight_kg":60,"reps":8}]}]}}"#;
    let request = server
        .mock("POST", "/v1/routines")
        .match_header("api-key", "api-key")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::JsonString(payload.to_owned()))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"routine-1","title":"Upper","exercises":[]}"#)
        .create();
    let update_request = server
        .mock("PUT", "/v1/routines/routine-1")
        .match_header("api-key", "api-key")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::JsonString(
            r#"{"routine":{"title":"Upper revised","exercises":[]}}"#.to_owned(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"routine-1","title":"Upper revised","exercises":[]}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routines",
            "create",
            "--data",
            payload,
        ])
        .assert()
        .success()
        .stdout("{\"exercises\":[],\"id\":\"routine-1\",\"title\":\"Upper\"}\n")
        .stderr("");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routines",
            "update",
            "routine-1",
            "--data",
            r#"{"routine":{"title":"Upper revised","exercises":[]}}"#,
        ])
        .assert()
        .success()
        .stdout("{\"exercises\":[],\"id\":\"routine-1\",\"title\":\"Upper revised\"}\n")
        .stderr("");
    command(&server, &config_home)
        .args([
            "--format", "json", "routines", "update", "routine-1", "--dry-run", "--data",
            r#"{"routine":{"title":"Upper","exercises":[]}}"#,
        ])
        .assert()
        .success()
        .stdout("{\"affected_resource\":\"routine-1\",\"dry_run\":true,\"request\":{\"body\":{\"routine\":{\"exercises\":[],\"title\":\"Upper\"}},\"method\":\"PUT\",\"path\":\"/v1/routines/routine-1\"}}\n")
        .stderr("");

    request.assert();
    update_request.assert();
}

#[test]
fn routine_mutations_reject_unwrapped_payloads_before_dry_runs_or_requests() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let create_request = server.mock("POST", "/v1/routines").expect(0).create();
    let update_request = server
        .mock("PUT", "/v1/routines/routine-1")
        .expect(0)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routines",
            "create",
            "--data",
            r#"{"title":"Unwrapped","exercises":[]}"#,
        ])
        .assert()
        .code(2)
        .stdout("")
        .stderr("{\"code\":\"invocation\",\"message\":\"Routine payload must contain a top-level \\\"routine\\\" object.\"}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "routines",
            "update",
            "routine-1",
            "--dry-run",
            "--data",
            r#"{"title":"Unwrapped","exercises":[]}"#,
        ])
        .assert()
        .code(2)
        .stdout("")
        .stderr("{\"code\":\"invocation\",\"message\":\"Routine payload must contain a top-level \\\"routine\\\" object.\"}\n");

    create_request.assert();
    update_request.assert();
}

#[test]
fn routine_mutations_reject_response_only_fields_with_json_paths_before_requests() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let create_request = server.mock("POST", "/v1/routines").expect(0).create();
    let update_request = server
        .mock("PUT", "/v1/routines/routine-1")
        .expect(0)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "routines",
            "create",
            "--dry-run",
            "--data",
            r#"{"routine":{"title":"Upper","exercises":[{"index":0,"exercise_template_id":"bench","sets":[]}]}}"#,
        ])
        .assert()
        .code(2)
        .stdout("")
        .stderr("{\"code\":\"invocation\",\"message\":\"Invalid routine create payload: routine.exercises[0].index is not accepted; omit response-only fields.\"}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "routines",
            "update",
            "routine-1",
            "--dry-run",
            "--data",
            r#"{"routine":{"title":"Upper","exercises":[{"exercise_template_id":"bench","sets":[{"index":0,"type":"normal"}]}]}}"#,
        ])
        .assert()
        .code(2)
        .stdout("")
        .stderr("{\"code\":\"invocation\",\"message\":\"Invalid routine update payload: routine.exercises[0].sets[0].index is not accepted; omit response-only fields.\"}\n");

    create_request.assert();
    update_request.assert();
}

#[test]
fn routine_create_and_update_enforce_their_distinct_note_nullability() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let create_request = server.mock("POST", "/v1/routines").expect(0).create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "routines",
            "create",
            "--dry-run",
            "--data",
            r#"{"routine":{"title":"Upper","notes":null,"exercises":[]}}"#,
        ])
        .assert()
        .code(2)
        .stdout("")
        .stderr("{\"code\":\"invocation\",\"message\":\"Invalid routine create payload: routine.notes: invalid type: null, expected a string.\"}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "routines",
            "update",
            "routine-1",
            "--dry-run",
            "--data",
            r#"{"routine":{"title":"Upper","notes":null,"exercises":[]}}"#,
        ])
        .assert()
        .success()
        .stdout("{\"affected_resource\":\"routine-1\",\"dry_run\":true,\"request\":{\"body\":{\"routine\":{\"exercises\":[],\"notes\":null,\"title\":\"Upper\"}},\"method\":\"PUT\",\"path\":\"/v1/routines/routine-1\"}}\n")
        .stderr("");

    create_request.assert();
}

#[test]
fn routine_api_failures_include_a_bounded_hevy_error_detail() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("PUT", "/v1/routines/routine-1")
        .match_header("api-key", "api-key")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":"routine is required"}"#)
        .create();

    let output = command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routines",
            "update",
            "routine-1",
            "--data",
            r#"{"routine":{"title":"Upper","exercises":[]}}"#,
        ])
        .assert()
        .code(4)
        .get_output()
        .clone();

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stderr).unwrap(),
        serde_json::json!({
            "code": "api",
            "message": "The Hevy API request failed: routine is required.",
            "status": 400
        })
    );
    request.assert();
}

#[test]
fn routine_api_failures_ignore_oversized_hevy_error_details() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("PUT", "/v1/routines/routine-1")
        .match_header("api-key", "api-key")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(serde_json::json!({ "error": "x".repeat(501) }).to_string())
        .create();

    let output = command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routines",
            "update",
            "routine-1",
            "--data",
            r#"{"routine":{"title":"Upper","exercises":[]}}"#,
        ])
        .assert()
        .code(4)
        .get_output()
        .clone();

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stderr).unwrap(),
        serde_json::json!({
            "code": "api",
            "message": "The Hevy API request failed.",
            "status": 400
        })
    );
    request.assert();
}

#[test]
fn routine_mutation_transport_failure_is_outcome_unknown_without_retrying() {
    let config_home = TempDir::new().unwrap();
    let (base_url, server) = dropping_server();
    let mut command = Command::cargo_bin("hevy-rs").unwrap();

    let output = command
        .env("HEVY_API_BASE_URL", base_url)
        .env("HEVY_CONFIG_DIR", config_home.path())
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routines",
            "create",
            "--data",
            r#"{"routine":{"title":"Ambiguous","exercises":[]}}"#,
        ])
        .assert()
        .code(5)
        .get_output()
        .clone();

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stderr).unwrap(),
        serde_json::json!({
            "code": "transport",
            "message": "The routine mutation outcome is unknown. Reconcile the affected routine before retrying."
        })
    );
    assert_eq!(server.join().unwrap(), 1, "the mutation must not retry");
}

#[test]
fn routines_all_retrieves_every_page_and_rejects_an_explicit_page() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let first_page = server
        .mock("GET", "/v1/routines")
        .match_query(mockito::Matcher::UrlEncoded("page".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":2,"routines":[{"id":"routine-1"}]}"#)
        .create();
    let second_page = server
        .mock("GET", "/v1/routines")
        .match_query(mockito::Matcher::UrlEncoded("page".into(), "2".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":2,"page_count":2,"routines":[{"id":"routine-2"}]}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format", "json", "--api-key", "api-key", "routines", "list", "--all",
        ])
        .assert()
        .success()
        .stdout("{\"all\":true,\"items\":[{\"id\":\"routine-1\"},{\"id\":\"routine-2\"}],\"page\":1,\"page_count\":2,\"pages_fetched\":[1,2]}\n")
        .stderr("");
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "routines",
            "list",
            "--all",
            "--page",
            "1",
        ])
        .assert()
        .code(2);

    first_page.assert();
    second_page.assert();
}

fn dropping_server() -> (String, thread::JoinHandle<usize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut request = Vec::new();
        stream.read_to_end(&mut request).unwrap_or_default();
        assert!(
            String::from_utf8_lossy(&request).starts_with("POST /v1/routines HTTP/"),
            "fixture must receive the mutation request"
        );
        drop(stream);

        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_millis(250);
        let mut requests = 1;
        while Instant::now() < deadline {
            match listener.accept() {
                Ok((stream, _)) => {
                    requests += 1;
                    drop(stream);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("could not accept fixture request: {error}"),
            }
        }
        requests
    });
    (format!("http://{address}"), server)
}
