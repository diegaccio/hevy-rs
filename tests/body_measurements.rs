use assert_cmd::Command;
use mockito::Server;
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
fn body_measurement_commands_use_documented_requests() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let list = server
        .mock("GET", "/v1/body_measurements")
        .match_header("api-key", "api-key")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("page".into(), "2".into()),
            mockito::Matcher::UrlEncoded("pageSize".into(), "10".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":2,"page_count":2,"body_measurements":[{"date":"2025-01-15","weight_kg":80.5}]}"#)
        .create();
    let get = server
        .mock("GET", "/v1/body_measurements/2025-01-15")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"date":"2025-01-15","weight_kg":80.5}"#)
        .create();
    let create = server
        .mock("POST", "/v1/body_measurements")
        .match_header("api-key", "api-key")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::JsonString(
            r#"{"date":"2025-01-15","weight_kg":80.5}"#.to_owned(),
        ))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"date":"2025-01-15","weight_kg":80.5}"#)
        .create();
    let update = server
        .mock("PUT", "/v1/body_measurements/2025-01-15")
        .match_header("api-key", "api-key")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::JsonString(
            r#"{"weight_kg":81.0,"body_fat_percentage":null}"#.to_owned(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"date":"2025-01-15","weight_kg":81.0,"body_fat_percentage":null}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "body-measurements",
            "list",
            "--page",
            "2",
            "--page-size",
            "10",
        ])
        .assert()
        .success()
        .stdout("{\"items\":[{\"date\":\"2025-01-15\",\"weight_kg\":80.5}],\"page\":2,\"page_count\":2}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "body-measurements",
            "get",
            "2025-01-15",
        ])
        .assert()
        .success()
        .stdout("{\"date\":\"2025-01-15\",\"weight_kg\":80.5}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "body-measurements",
            "create",
            "--data",
            r#"{"date":"2025-01-15","weight_kg":80.5}"#,
        ])
        .assert()
        .success();
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "body-measurements",
            "update",
            "2025-01-15",
            "--data",
            r#"{"weight_kg":81.0,"body_fat_percentage":null}"#,
        ])
        .assert()
        .success();

    list.assert();
    get.assert();
    create.assert();
    update.assert();
}

#[test]
fn body_measurement_list_has_readable_default_output() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let request = server
        .mock("GET", "/v1/body_measurements")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":1,"body_measurements":[{"date":"2025-01-15","weight_kg":80.5}]}"#)
        .create();

    command(&server, &config_home)
        .args(["--api-key", "api-key", "body-measurements", "list"])
        .assert()
        .success()
        .stdout("Page: 1 of 1\n- 2025-01-15\n")
        .stderr("");

    request.assert();
}

#[test]
fn body_measurement_dates_and_pagination_are_validated_locally() {
    let server = Server::new();
    let config_home = TempDir::new().unwrap();

    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "body-measurements",
            "get",
            "2025-02-29",
        ])
        .assert()
        .code(2);
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "body-measurements",
            "update",
            "not-a-date",
            "--data",
            "{}",
        ])
        .assert()
        .code(2);
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "body-measurements",
            "list",
            "--page",
            "0",
        ])
        .assert()
        .code(2);
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "body-measurements",
            "list",
            "--page-size",
            "11",
        ])
        .assert()
        .code(2);
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "body-measurements",
            "list",
            "--all",
            "--page",
            "1",
        ])
        .assert()
        .code(2);
}

#[test]
fn body_measurements_all_retrieves_every_page() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let first_page = server
        .mock("GET", "/v1/body_measurements")
        .match_query(mockito::Matcher::UrlEncoded("page".into(), "1".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":2,"body_measurements":[{"date":"2025-01-15"}]}"#)
        .create();
    let second_page = server
        .mock("GET", "/v1/body_measurements")
        .match_query(mockito::Matcher::UrlEncoded("page".into(), "2".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":2,"page_count":2,"body_measurements":[{"date":"2025-01-16"}]}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "body-measurements",
            "list",
            "--all",
        ])
        .assert()
        .success()
        .stdout("{\"all\":true,\"items\":[{\"date\":\"2025-01-15\"},{\"date\":\"2025-01-16\"}],\"page\":1,\"page_count\":2,\"pages_fetched\":[1,2]}\n")
        .stderr("");

    first_page.assert();
    second_page.assert();
}

#[test]
fn body_measurement_writes_support_dry_run_and_explain_conflicts() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let conflict = server
        .mock("POST", "/v1/body_measurements")
        .with_status(409)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":"A body measurement already exists for this date"}"#)
        .create();

    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "body-measurements",
            "update",
            "2025-01-15",
            "--dry-run",
            "--data",
            r#"{"weight_kg":81.0,"api_key":"secret"}"#,
        ])
        .assert()
        .success()
        .stdout("{\"affected_resource\":\"2025-01-15\",\"dry_run\":true,\"request\":{\"body\":{\"api_key\":\"[REDACTED]\",\"weight_kg\":81.0},\"method\":\"PUT\",\"path\":\"/v1/body_measurements/2025-01-15\"}}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "body-measurements",
            "create",
            "--data",
            r#"{"date":"2025-01-15","weight_kg":80.5}"#,
        ])
        .assert()
        .code(4)
        .stderr("{\"code\":\"api\",\"message\":\"A body measurement already exists for that date. Retrieve it and use update to replace all measurement fields.\",\"status\":409}\n");

    conflict.assert();
}
