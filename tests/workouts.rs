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
        .env("HEVY_API_BASE_URL", server.url())
        .env("HEVY_CONFIG_DIR", config_home.path())
        .env("XDG_CONFIG_HOME", config_home.path())
        .env("APPDATA", config_home.path())
        .env("LOCALAPPDATA", config_home.path())
        .env("HOME", config_home.path());
    command
}

#[test]
fn workouts_list_makes_the_documented_paginated_request_and_normalizes_json() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/workouts")
        .match_header("api-key", "api-key")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "2".into()),
            mockito::Matcher::UrlEncoded("pageSize".into(), "10".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":2,"page_count":3,"workouts":[{"id":"workout-1","title":"Morning"}]}"#)
        .create();

    let output = command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "workouts",
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
            "items": [{"id":"workout-1","title":"Morning"}],
            "page": 2,
            "page_count": 3
        })
    );
    assert!(output.stderr.is_empty());
    request.assert();
}

#[test]
fn workouts_list_has_readable_default_output() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/workouts")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":1,"workouts":[{"id":"workout-1","title":"Morning"}]}"#)
        .create();

    let output = command(&server, &config_home)
        .args(["--api-key", "api-key", "workouts", "list"])
        .assert()
        .success()
        .get_output()
        .clone();

    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("Morning")
    );
    assert!(output.stderr.is_empty());
    request.assert();
}

#[test]
fn workouts_count_and_get_make_documented_requests() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let count_request = server
        .mock("GET", "/v1/workouts/count")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"workout_count":42}"#)
        .create();
    let get_request = server
        .mock("GET", "/v1/workouts/workout-123")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"workout-123","title":"Strength"}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "workouts",
            "count",
        ])
        .assert()
        .success()
        .stdout("{\"workout_count\":42}\n")
        .stderr("");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "workouts",
            "get",
            "workout-123",
        ])
        .assert()
        .success()
        .stdout("{\"id\":\"workout-123\",\"title\":\"Strength\"}\n")
        .stderr("");

    count_request.assert();
    get_request.assert();
}

#[test]
fn workout_get_has_a_concise_readable_summary() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/workouts/workout-1")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"workout-1","title":"Lower strength","start_time":"2026-07-30T05:42:48Z","end_time":"2026-07-30T06:52:52Z","exercises":[{"title":"Box Jump","sets":[{},{}]},{"title":"Squat","sets":[{}]}]}"#)
        .create();

    command(&server, &config_home)
        .args(["--api-key", "api-key", "workouts", "get", "workout-1"])
        .assert()
        .success()
        .stdout("ID: workout-1\nTitle: Lower strength\nStarted: 2026-07-30T05:42:48Z\nEnded: 2026-07-30T06:52:52Z\nExercises:\n- Box Jump (2 sets)\n- Squat (1 set)\n")
        .stderr("");

    request.assert();
}

#[test]
fn workouts_count_has_readable_default_output() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/workouts/count")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"workout_count":42}"#)
        .create();

    command(&server, &config_home)
        .args(["--api-key", "api-key", "workouts", "count"])
        .assert()
        .success()
        .stdout("Workout count: 42\n")
        .stderr("");

    request.assert();
}

#[test]
fn workouts_events_validates_since_and_retrieves_events() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/workouts/events")
        .match_header("api-key", "api-key")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "1".into()),
            mockito::Matcher::UrlEncoded("pageSize".into(), "5".into()),
            mockito::Matcher::UrlEncoded("since".into(), "2025-01-01T00:00:00Z".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":1,"events":[{"type":"updated","id":"workout-1"}]}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "workouts",
            "events",
            "--page",
            "1",
            "--page-size",
            "5",
            "--since",
            "2025-01-01T00:00:00Z",
        ])
        .assert()
        .success()
        .stdout("{\"items\":[{\"id\":\"workout-1\",\"type\":\"updated\"}],\"page\":1,\"page_count\":1}\n")
        .stderr("");
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "workouts",
            "events",
            "--since",
            "yesterday",
        ])
        .assert()
        .code(2);

    request.assert();
}

#[test]
fn workout_events_render_updated_and_deleted_workouts_readably() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/workouts/events")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":1,"events":[{"type":"updated","workout":{"id":"workout-1","title":"Lower body"}},{"type":"deleted","id":"workout-2"}]}"#)
        .create();

    command(&server, &config_home)
        .args(["--api-key", "api-key", "workouts", "events"])
        .assert()
        .success()
        .stdout("Page: 1 of 1\n- Updated: Lower body (workout-1)\n- Deleted: workout-2\n")
        .stderr("");

    request.assert();
}

#[test]
fn workouts_all_retrieves_every_page_and_rejects_an_explicit_page() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let first_page = server
        .mock("GET", "/v1/workouts")
        .match_query(mockito::Matcher::UrlEncoded("page".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":2,"workouts":[{"id":"workout-1"}]}"#)
        .create();
    let second_page = server
        .mock("GET", "/v1/workouts")
        .match_query(mockito::Matcher::UrlEncoded("page".into(), "2".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":2,"page_count":2,"workouts":[{"id":"workout-2"}]}"#)
        .create();

    command(&server, &config_home)
        .args(["--format", "json", "--api-key", "api-key", "workouts", "list", "--all"])
        .assert()
        .success()
        .stdout("{\"all\":true,\"items\":[{\"id\":\"workout-1\"},{\"id\":\"workout-2\"}],\"page\":1,\"page_count\":2,\"pages_fetched\":[1,2]}\n")
        .stderr("");
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "workouts",
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

#[test]
fn workout_events_all_preserves_the_since_boundary_on_every_page() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let first_page = server
        .mock("GET", "/v1/workouts/events")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "1".into()),
            mockito::Matcher::UrlEncoded("since".into(), "2025-01-01T00:00:00Z".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":2,"events":[{"id":"event-1"}]}"#)
        .create();
    let second_page = server
        .mock("GET", "/v1/workouts/events")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "2".into()),
            mockito::Matcher::UrlEncoded("since".into(), "2025-01-01T00:00:00Z".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":2,"page_count":2,"events":[{"id":"event-2"}]}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "workouts",
            "events",
            "--all",
            "--since",
            "2025-01-01T00:00:00Z",
        ])
        .assert()
        .success()
        .stdout("{\"all\":true,\"items\":[{\"id\":\"event-1\"},{\"id\":\"event-2\"}],\"page\":1,\"page_count\":2,\"pages_fetched\":[1,2]}\n")
        .stderr("");

    first_page.assert();
    second_page.assert();
}

#[test]
fn workout_pagination_is_validated_locally() {
    let server = Server::new();
    let config_home = TempDir::new().unwrap();

    command(&server, &config_home)
        .args(["--api-key", "api-key", "workouts", "list", "--page", "0"])
        .assert()
        .code(2);
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "workouts",
            "events",
            "--page-size",
            "11",
        ])
        .assert()
        .code(2);
}

#[test]
fn workout_create_sends_an_inline_complete_payload() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("POST", "/v1/workouts")
        .match_header("api-key", "api-key")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::JsonString(r#"{"title":"Morning","exercises":[{"exercise_template_id":"squat","sets":[{"type":"normal","weight_kg":100,"reps":5}]}]}"#.to_owned()))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"workout-1","title":"Morning","exercises":[]}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format", "json", "--api-key", "api-key", "workouts", "create", "--data",
            r#"{"title":"Morning","exercises":[{"exercise_template_id":"squat","sets":[{"type":"normal","weight_kg":100,"reps":5}]}]}"#,
        ])
        .assert()
        .success()
        .stdout("{\"exercises\":[],\"id\":\"workout-1\",\"title\":\"Morning\"}\n")
        .stderr("");

    request.assert();
}

#[test]
fn workout_update_sends_a_file_payload() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let payload_path = config_home.path().join("workout.json");
    std::fs::write(&payload_path, r#"{"title":"Evening","exercises":[]}"#).unwrap();
    let request = server
        .mock("PUT", "/v1/workouts/workout-1")
        .match_header("api-key", "api-key")
        .match_body(mockito::Matcher::JsonString(
            r#"{"title":"Evening","exercises":[]}"#.to_owned(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"workout-1","title":"Evening","exercises":[]}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "workouts",
            "update",
            "workout-1",
            "--data",
            &format!("@{}", payload_path.display()),
        ])
        .assert()
        .success();

    request.assert();
}

#[test]
fn workout_mutations_accept_standard_input_payloads() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let create_request = server
        .mock("POST", "/v1/workouts")
        .match_body(mockito::Matcher::JsonString(
            r#"{"title":"From stdin","exercises":[]}"#.to_owned(),
        ))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"workout-2"}"#)
        .create();
    let update_request = server
        .mock("PUT", "/v1/workouts/workout-2")
        .match_body(mockito::Matcher::JsonString(
            r#"{"title":"Updated stdin","exercises":[]}"#.to_owned(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"workout-2"}"#)
        .create();

    command(&server, &config_home)
        .args(["--api-key", "api-key", "workouts", "create", "--data", "-"])
        .write_stdin(r#"{"title":"From stdin","exercises":[]}"#)
        .assert()
        .success();
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "workouts",
            "update",
            "workout-2",
            "--data",
            "-",
        ])
        .write_stdin(r#"{"title":"Updated stdin","exercises":[]}"#)
        .assert()
        .success();

    create_request.assert();
    update_request.assert();
}

#[test]
fn workout_mutation_dry_run_validates_and_redacts_without_a_request() {
    let server = Server::new();
    let config_home = TempDir::new().unwrap();

    command(&server, &config_home)
        .args([
            "--format", "json", "workouts", "update", "workout-1", "--dry-run", "--data",
            r#"{"title":"Safe","api_key":"secret","nested":{"authorization":"Bearer secret"},"exercises":[]}"#,
        ])
        .assert()
        .success()
        .stdout("{\"affected_resource\":\"workout-1\",\"dry_run\":true,\"request\":{\"body\":{\"api_key\":\"[REDACTED]\",\"exercises\":[],\"nested\":{\"authorization\":\"[REDACTED]\"},\"title\":\"Safe\"},\"method\":\"PUT\",\"path\":\"/v1/workouts/workout-1\"}}\n")
        .stderr("");

    command(&server, &config_home)
        .args(["workouts", "create", "--dry-run", "--yes", "--data", "{}"])
        .assert()
        .code(2);
    command(&server, &config_home)
        .args(["workouts", "create", "--dry-run", "--data", "not json"])
        .assert()
        .code(2);
}

#[test]
fn workout_mutation_transport_failure_is_outcome_unknown_without_retrying() {
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
            "workouts",
            "create",
            "--data",
            r#"{"title":"Ambiguous","exercises":[]}"#,
        ])
        .assert()
        .code(5)
        .get_output()
        .clone();

    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stderr).unwrap(),
        serde_json::json!({
            "code": "transport",
            "message": "The workout mutation outcome is unknown. Reconcile the affected workout before retrying."
        })
    );
    assert_eq!(server.join().unwrap(), 1, "the mutation must not retry");
}

#[test]
fn workout_mutation_http_failure_is_not_retried() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("POST", "/v1/workouts")
        .with_status(503)
        .expect(1)
        .create();

    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "workouts",
            "create",
            "--data",
            r#"{"title":"Unavailable","exercises":[]}"#,
        ])
        .assert()
        .code(4);

    request.assert();
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
            String::from_utf8_lossy(&request).starts_with("POST /v1/workouts HTTP/"),
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
