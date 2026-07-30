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
fn exercise_template_commands_use_documented_requests() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let list = server
        .mock("GET", "/v1/exercise_templates")
        .match_header("api-key", "api-key")
        .match_query(mockito::Matcher::UrlEncoded("pageSize".into(), "100".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"page":1,"page_count":1,"exercise_templates":[{"id":"template-1","title":"Squat"}]}"#)
        .create();
    let get = server
        .mock("GET", "/v1/exercise_templates/template-1")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"template-1","title":"Squat"}"#)
        .create();
    let create = server
        .mock("POST", "/v1/exercise_templates")
        .match_header("api-key", "api-key")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::JsonString(
            r#"{"exercise":{"title":"Custom Squat","exercise_type":"weight_reps"}}"#.to_owned(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"template-2"}"#)
        .create();

    command(&server, &config_home)
        .args(["--format", "json", "--api-key", "api-key", "exercise-templates", "list", "--page-size", "100"])
        .assert()
        .success()
        .stdout("{\"items\":[{\"id\":\"template-1\",\"title\":\"Squat\"}],\"page\":1,\"page_count\":1}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "exercise-templates",
            "get",
            "template-1",
        ])
        .assert()
        .success()
        .stdout("{\"id\":\"template-1\",\"title\":\"Squat\"}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "exercise-templates",
            "create",
            "--data",
            r#"{"exercise":{"title":"Custom Squat","exercise_type":"weight_reps"}}"#,
        ])
        .assert()
        .success()
        .stdout("{\"id\":\"template-2\"}\n");

    list.assert();
    get.assert();
    create.assert();
}

#[test]
fn routine_folder_commands_use_documented_requests() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let list = server
        .mock("GET", "/v1/routine_folders")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"page":1,"page_count":1,"routine_folders":[{"id":"folder-1","title":"Strength"}]}"#,
        )
        .create();
    let get = server
        .mock("GET", "/v1/routine_folders/folder-1")
        .match_header("api-key", "api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"folder-1","title":"Strength"}"#)
        .create();
    let create = server
        .mock("POST", "/v1/routine_folders")
        .match_header("api-key", "api-key")
        .match_body(mockito::Matcher::JsonString(
            r#"{"routine_folder":{"title":"Hypertrophy"}}"#.to_owned(),
        ))
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"folder-2","title":"Hypertrophy"}"#)
        .create();

    command(&server, &config_home)
        .args(["--format", "json", "--api-key", "api-key", "routine-folders", "list"])
        .assert()
        .success()
        .stdout("{\"items\":[{\"id\":\"folder-1\",\"title\":\"Strength\"}],\"page\":1,\"page_count\":1}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routine-folders",
            "get",
            "folder-1",
        ])
        .assert()
        .success()
        .stdout("{\"id\":\"folder-1\",\"title\":\"Strength\"}\n");
    command(&server, &config_home)
        .args([
            "--format",
            "json",
            "--api-key",
            "api-key",
            "routine-folders",
            "create",
            "--data",
            r#"{"routine_folder":{"title":"Hypertrophy"}}"#,
        ])
        .assert()
        .success()
        .stdout("{\"id\":\"folder-2\",\"title\":\"Hypertrophy\"}\n");

    list.assert();
    get.assert();
    create.assert();
}

#[test]
fn exercise_history_passes_validated_optional_bounds_to_the_documented_endpoint() {
    let mut server = Server::new();
    let config_home = TempDir::new().unwrap();
    let history = server
        .mock("GET", "/v1/exercise_history/template-1")
        .match_header("api-key", "api-key")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("start_date".into(), "2025-01-01T00:00:00Z".into()),
            mockito::Matcher::UrlEncoded("end_date".into(), "2025-01-31T23:59:59Z".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"exercise_history":[{"workout_id":"workout-1","sets":[{"reps":5,"weight_kg":100}]}]}"#)
        .create();

    command(&server, &config_home)
        .args(["--format", "json", "--api-key", "api-key", "exercise-history", "get", "template-1", "--start", "2025-01-01T00:00:00Z", "--end", "2025-01-31T23:59:59Z"])
        .assert()
        .success()
        .stdout("{\"exercise_history\":[{\"sets\":[{\"reps\":5,\"weight_kg\":100}],\"workout_id\":\"workout-1\"}]}\n");
    command(&server, &config_home)
        .args([
            "--api-key",
            "api-key",
            "exercise-history",
            "get",
            "template-1",
            "--start",
            "yesterday",
        ])
        .assert()
        .code(2);

    history.assert();
}
