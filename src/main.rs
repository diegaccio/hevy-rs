mod client;
mod error;
mod render;

use clap::{Args, Parser, Subcommand, error::ErrorKind};
use error::AppError;
use render::OutputFormat;
use serde::Deserialize;
use std::{
    env, fs,
    io::{self, Read},
    process,
};
use time::{Date, OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Parser)]
#[command(
    name = "hevy-rs",
    version,
    about = "A command-line client for the Hevy API"
)]
struct Cli {
    /// API key. Prefer HEVY_API_KEY or the per-user configuration file in automation.
    #[arg(long, env = "HEVY_API_KEY", hide_env_values = true, global = true)]
    api_key: Option<String>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, global = true)]
    format: OutputFormat,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Commands for the authenticated Hevy user.
    User {
        #[command(subcommand)]
        command: UserCommand,
    },
    /// Commands for workouts.
    Workouts {
        #[command(subcommand)]
        command: WorkoutCommand,
    },
    /// Commands for routines.
    Routines {
        #[command(subcommand)]
        command: RoutineCommand,
    },
    /// Commands for exercise templates.
    ExerciseTemplates {
        #[command(subcommand)]
        command: ExerciseTemplateCommand,
    },
    /// Commands for routine folders.
    RoutineFolders {
        #[command(subcommand)]
        command: RoutineFolderCommand,
    },
    /// Commands for exercise history.
    ExerciseHistory {
        #[command(subcommand)]
        command: ExerciseHistoryCommand,
    },
    /// Commands for dated body measurements.
    BodyMeasurements {
        #[command(subcommand)]
        command: BodyMeasurementCommand,
    },
}

#[derive(Subcommand)]
enum UserCommand {
    /// Retrieve the authenticated user's information.
    Get,
}

#[derive(Subcommand)]
enum RoutineCommand {
    /// List routines.
    List(PaginationArgs),
    /// Retrieve a routine's complete details.
    Get { routine_id: String },
    /// Export a routine as a validated update request payload.
    ExportUpdatePayload {
        /// Routine identifier.
        routine_id: String,
    },
    /// Create a routine from a complete API-shaped JSON request body with a top-level `routine` object.
    Create(MutationArgs),
    /// Replace a routine with a complete API-shaped JSON request body with a top-level `routine` object.
    Update {
        /// Routine identifier.
        routine_id: String,
        #[command(flatten)]
        mutation: MutationArgs,
    },
}

#[derive(Subcommand)]
enum ExerciseTemplateCommand {
    /// List exercise templates.
    List(ExerciseTemplatePaginationArgs),
    /// Retrieve an exercise template.
    Get { exercise_template_id: String },
    /// Create an exercise template from a complete API-shaped JSON payload.
    Create(MutationArgs),
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PostRoutineRequest {
    routine: PostRoutine,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PostRoutine {
    title: Option<String>,
    folder_id: Option<f64>,
    #[serde(default)]
    notes: PostRoutineNotes,
    exercises: Option<Vec<PostRoutineExercise>>,
}

#[allow(dead_code)]
struct PostRoutineNotes;

impl<'de> Deserialize<'de> for PostRoutineNotes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|_| Self)
    }
}

impl Default for PostRoutineNotes {
    fn default() -> Self {
        Self
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PostRoutineExercise {
    exercise_template_id: Option<String>,
    superset_id: Option<i64>,
    rest_seconds: Option<i64>,
    notes: Option<String>,
    sets: Option<Vec<PostRoutineSet>>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PostRoutineSet {
    #[serde(rename = "type")]
    kind: Option<RoutineSetType>,
    weight_kg: Option<f64>,
    reps: Option<i64>,
    distance_meters: Option<i64>,
    duration_seconds: Option<i64>,
    custom_metric: Option<f64>,
    rep_range: Option<RepRange>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PutRoutineRequest {
    routine: PutRoutine,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PutRoutine {
    title: Option<String>,
    folder_id: Option<f64>,
    notes: Option<String>,
    exercises: Option<Vec<PutRoutineExercise>>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PutRoutineExercise {
    exercise_template_id: Option<String>,
    superset_id: Option<i64>,
    rest_seconds: Option<i64>,
    notes: Option<String>,
    sets: Option<Vec<PutRoutineSet>>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PutRoutineSet {
    #[serde(rename = "type")]
    kind: Option<RoutineSetType>,
    weight_kg: Option<f64>,
    reps: Option<i64>,
    distance_meters: Option<i64>,
    duration_seconds: Option<i64>,
    custom_metric: Option<f64>,
    rep_range: Option<RepRange>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RepRange {
    start: Option<f64>,
    end: Option<f64>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum RoutineSetType {
    Warmup,
    Normal,
    Failure,
    Dropset,
}

#[derive(Subcommand)]
enum RoutineFolderCommand {
    /// List routine folders.
    List(PaginationArgs),
    /// Retrieve a routine folder.
    Get { folder_id: String },
    /// Create a routine folder from a complete API-shaped JSON payload.
    Create(MutationArgs),
}

#[derive(Subcommand)]
enum ExerciseHistoryCommand {
    /// Retrieve an exercise template's history.
    Get {
        /// Exercise template identifier.
        exercise_template_id: String,
        /// Include history on or after this ISO-8601 timestamp.
        #[arg(long, value_parser = parse_iso8601)]
        start: Option<String>,
        /// Include history on or before this ISO-8601 timestamp.
        #[arg(long, value_parser = parse_iso8601)]
        end: Option<String>,
    },
}

#[derive(Subcommand)]
enum BodyMeasurementCommand {
    /// List body measurements.
    List(PaginationArgs),
    /// Retrieve a body measurement by date.
    Get {
        /// Measurement date in YYYY-MM-DD form.
        #[arg(value_parser = parse_date)]
        date: String,
    },
    /// Create a body measurement from a complete API-shaped JSON payload.
    Create(MutationArgs),
    /// Replace a body measurement with a complete API-shaped JSON payload.
    Update {
        /// Measurement date in YYYY-MM-DD form.
        #[arg(value_parser = parse_date)]
        date: String,
        #[command(flatten)]
        mutation: MutationArgs,
    },
}

#[derive(Subcommand)]
enum WorkoutCommand {
    /// List workouts.
    List(PaginationArgs),
    /// Count workouts.
    Count,
    /// Retrieve a workout's complete details.
    Get { workout_id: String },
    /// Retrieve workout change events.
    Events {
        #[command(flatten)]
        pagination: PaginationArgs,
        /// Return events since this ISO-8601 timestamp.
        #[arg(long, value_parser = parse_iso8601)]
        since: Option<String>,
    },
    /// Create a workout from a complete API-shaped JSON payload.
    Create(MutationArgs),
    /// Replace a workout with a complete API-shaped JSON payload.
    Update {
        /// Workout identifier.
        workout_id: String,
        #[command(flatten)]
        mutation: MutationArgs,
    },
}

#[derive(Args)]
struct MutationArgs {
    /// Complete API-shaped JSON payload, a file path prefixed with @, or - for standard input.
    #[arg(long)]
    data: String,

    /// Validate and display the intended request without sending it.
    #[arg(long)]
    dry_run: bool,

    /// Confirm an irreversible operation when one is available.
    #[arg(long)]
    yes: bool,
}

#[derive(Args)]
struct PaginationArgs {
    /// Page number, starting at 1.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..), conflicts_with = "all")]
    page: Option<u32>,

    /// Number of items per page (maximum 10).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=10))]
    page_size: Option<u32>,

    /// Retrieve every page.
    #[arg(long)]
    all: bool,
}

#[derive(Args)]
struct ExerciseTemplatePaginationArgs {
    /// Page number, starting at 1.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..), conflicts_with = "all")]
    page: Option<u32>,

    /// Number of items per page (maximum 100).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=100))]
    page_size: Option<u32>,

    /// Retrieve every page.
    #[arg(long)]
    all: bool,
}

#[derive(Deserialize)]
struct Config {
    api_key: Option<String>,
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            error.print().expect("help output is writable");
            process::exit(0);
        }
        Err(error) => exit_with(
            AppError::invocation(error.to_string()),
            if requests_json_output() {
                OutputFormat::Json
            } else {
                OutputFormat::Text
            },
        ),
    };

    let format = cli.format;
    let result = match cli.command {
        Command::User {
            command: UserCommand::Get,
        } => resolve_api_key(cli.api_key).and_then(|api_key| client::get_user(&api_key)),
        Command::Workouts { command } => match command {
            WorkoutCommand::List(pagination) => resolve_api_key(cli.api_key).and_then(|api_key| {
                client::list_workouts(
                    &api_key,
                    client::Pagination {
                        page: pagination.page,
                        page_size: pagination.page_size,
                        all: pagination.all,
                    },
                )
            }),
            WorkoutCommand::Count => {
                resolve_api_key(cli.api_key).and_then(|api_key| client::get_workout_count(&api_key))
            }
            WorkoutCommand::Get { workout_id } => resolve_api_key(cli.api_key)
                .and_then(|api_key| client::get_workout(&api_key, &workout_id)),
            WorkoutCommand::Events { pagination, since } => {
                resolve_api_key(cli.api_key).and_then(|api_key| {
                    client::list_workout_events(
                        &api_key,
                        client::Pagination {
                            page: pagination.page,
                            page_size: pagination.page_size,
                            all: pagination.all,
                        },
                        since.as_deref(),
                    )
                })
            }
            WorkoutCommand::Create(mutation) => {
                execute_mutation(cli.api_key, "workouts", "workout", None, mutation)
            }
            WorkoutCommand::Update {
                workout_id,
                mutation,
            } => execute_mutation(
                cli.api_key,
                "workouts",
                "workout",
                Some(workout_id),
                mutation,
            ),
        },
        Command::ExerciseTemplates { command } => match command {
            ExerciseTemplateCommand::List(pagination) => {
                resolve_api_key(cli.api_key).and_then(|api_key| {
                    client::list_exercise_templates(
                        &api_key,
                        client::Pagination {
                            page: pagination.page,
                            page_size: pagination.page_size,
                            all: pagination.all,
                        },
                    )
                })
            }
            ExerciseTemplateCommand::Get {
                exercise_template_id,
            } => resolve_api_key(cli.api_key)
                .and_then(|api_key| client::get_exercise_template(&api_key, &exercise_template_id)),
            ExerciseTemplateCommand::Create(mutation) => execute_mutation(
                cli.api_key,
                "exercise_templates",
                "exercise template",
                None,
                mutation,
            ),
        },
        Command::RoutineFolders { command } => match command {
            RoutineFolderCommand::List(pagination) => {
                resolve_api_key(cli.api_key).and_then(|api_key| {
                    client::list_routine_folders(
                        &api_key,
                        client::Pagination {
                            page: pagination.page,
                            page_size: pagination.page_size,
                            all: pagination.all,
                        },
                    )
                })
            }
            RoutineFolderCommand::Get { folder_id } => resolve_api_key(cli.api_key)
                .and_then(|api_key| client::get_routine_folder(&api_key, &folder_id)),
            RoutineFolderCommand::Create(mutation) => execute_mutation(
                cli.api_key,
                "routine_folders",
                "routine folder",
                None,
                mutation,
            ),
        },
        Command::ExerciseHistory { command } => match command {
            ExerciseHistoryCommand::Get {
                exercise_template_id,
                start,
                end,
            } => resolve_api_key(cli.api_key).and_then(|api_key| {
                client::get_exercise_history(
                    &api_key,
                    &exercise_template_id,
                    start.as_deref(),
                    end.as_deref(),
                )
            }),
        },
        Command::BodyMeasurements { command } => match command {
            BodyMeasurementCommand::List(pagination) => {
                resolve_api_key(cli.api_key).and_then(|api_key| {
                    client::list_body_measurements(
                        &api_key,
                        client::Pagination {
                            page: pagination.page,
                            page_size: pagination.page_size,
                            all: pagination.all,
                        },
                    )
                })
            }
            BodyMeasurementCommand::Get { date } => resolve_api_key(cli.api_key)
                .and_then(|api_key| client::get_body_measurement(&api_key, &date)),
            BodyMeasurementCommand::Create(mutation) => execute_mutation(
                cli.api_key,
                "body_measurements",
                "body measurement",
                None,
                mutation,
            ),
            BodyMeasurementCommand::Update { date, mutation } => execute_mutation(
                cli.api_key,
                "body_measurements",
                "body measurement",
                Some(date),
                mutation,
            ),
        },
        Command::Routines { command } => match command {
            RoutineCommand::List(pagination) => resolve_api_key(cli.api_key).and_then(|api_key| {
                client::list_routines(
                    &api_key,
                    client::Pagination {
                        page: pagination.page,
                        page_size: pagination.page_size,
                        all: pagination.all,
                    },
                )
            }),
            RoutineCommand::Get { routine_id } => resolve_api_key(cli.api_key)
                .and_then(|api_key| client::get_routine(&api_key, &routine_id)),
            RoutineCommand::ExportUpdatePayload { routine_id } => {
                export_routine_update_payload(cli.api_key, &routine_id)
            }
            RoutineCommand::Create(mutation) => {
                execute_mutation(cli.api_key, "routines", "routine", None, mutation)
            }
            RoutineCommand::Update {
                routine_id,
                mutation,
            } => execute_mutation(
                cli.api_key,
                "routines",
                "routine",
                Some(routine_id),
                mutation,
            ),
        },
    };

    match result {
        Ok(user) => render::success(&user, format),
        Err(error) => exit_with(error, format),
    }
}

fn export_routine_update_payload(
    explicit_api_key: Option<String>,
    routine_id: &str,
) -> Result<serde_json::Value, AppError> {
    let api_key = resolve_api_key(explicit_api_key)?;
    let response = client::get_routine(&api_key, routine_id)?;
    let response = project_object(&response, "response", &["routine"], &[])?;
    let routine = response
        .get("routine")
        .ok_or_else(|| routine_projection_error("routine is missing"))?;
    let mut routine = project_object(
        routine,
        "routine",
        &["title", "folder_id", "notes", "exercises"],
        &["id", "created_at", "updated_at"],
    )?;

    if let Some(exercises) = routine.get_mut("exercises") {
        project_array(exercises, "routine.exercises", project_routine_exercise)?;
    }

    let payload = serde_json::json!({ "routine": routine });
    routine_request_validation_detail::<PutRoutineRequest>(&payload).map_err(|detail| {
        routine_projection_error(format!("the projected payload is invalid: {detail}"))
    })?;
    Ok(payload)
}

fn project_routine_exercise(
    exercise: &serde_json::Value,
    path: &str,
) -> Result<serde_json::Value, AppError> {
    let mut exercise = project_object(
        exercise,
        path,
        &[
            "exercise_template_id",
            "superset_id",
            "rest_seconds",
            "notes",
            "sets",
        ],
        &["index", "title"],
    )?;
    if let Some(sets) = exercise.get_mut("sets") {
        project_array(sets, &format!("{path}.sets"), project_routine_set)?;
    }
    Ok(serde_json::Value::Object(exercise))
}

fn project_routine_set(set: &serde_json::Value, path: &str) -> Result<serde_json::Value, AppError> {
    let mut set = project_object(
        set,
        path,
        &[
            "type",
            "weight_kg",
            "reps",
            "distance_meters",
            "duration_seconds",
            "custom_metric",
            "rep_range",
        ],
        &["index"],
    )?;
    if let Some(rep_range) = set.get_mut("rep_range")
        && !rep_range.is_null()
    {
        *rep_range = serde_json::Value::Object(project_object(
            rep_range,
            &format!("{path}.rep_range"),
            &["start", "end"],
            &[],
        )?);
    }
    Ok(serde_json::Value::Object(set))
}

fn project_array(
    value: &mut serde_json::Value,
    path: &str,
    project_item: fn(&serde_json::Value, &str) -> Result<serde_json::Value, AppError>,
) -> Result<(), AppError> {
    if value.is_null() {
        return Ok(());
    }
    let items = value
        .as_array()
        .ok_or_else(|| routine_projection_error(format!("{path} is not an array")))?;
    let projected = items
        .iter()
        .enumerate()
        .map(|(index, item)| project_item(item, &format!("{path}[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    *value = serde_json::Value::Array(projected);
    Ok(())
}

fn project_object(
    value: &serde_json::Value,
    path: &str,
    allowed_fields: &[&str],
    omitted_fields: &[&str],
) -> Result<serde_json::Map<String, serde_json::Value>, AppError> {
    let object = value
        .as_object()
        .ok_or_else(|| routine_projection_error(format!("{path} is not an object")))?;
    if let Some(field) = object.keys().find(|field| {
        !allowed_fields.contains(&field.as_str()) && !omitted_fields.contains(&field.as_str())
    }) {
        return Err(routine_projection_error(format!(
            "{path}.{field} is not recognized"
        )));
    }

    Ok(allowed_fields
        .iter()
        .filter_map(|field| {
            object
                .get(*field)
                .cloned()
                .map(|value| ((*field).to_owned(), value))
        })
        .collect())
}

fn routine_projection_error(detail: impl std::fmt::Display) -> AppError {
    AppError::api_message(format!(
        "The Hevy API response cannot be converted to a routine update payload: {detail}."
    ))
}

fn execute_mutation(
    explicit_api_key: Option<String>,
    resource_path: &str,
    resource_name: &str,
    resource_id: Option<String>,
    mutation: MutationArgs,
) -> Result<serde_json::Value, AppError> {
    if mutation.dry_run && mutation.yes {
        return Err(AppError::invocation(
            "--dry-run cannot be combined with --yes.",
        ));
    }

    let payload = read_payload(&mutation.data)?;
    validate_mutation_payload(resource_path, resource_id.is_some(), &payload)?;
    if mutation.dry_run {
        return Ok(dry_run_output(
            resource_path,
            resource_name,
            resource_id.as_deref(),
            payload,
        ));
    }

    let api_key = resolve_api_key(explicit_api_key)?;
    match (resource_path, resource_id) {
        ("workouts", Some(workout_id)) => client::update_workout(&api_key, &workout_id, &payload),
        ("workouts", None) => client::create_workout(&api_key, &payload),
        ("routines", Some(routine_id)) => client::update_routine(&api_key, &routine_id, &payload),
        ("routines", None) => client::create_routine(&api_key, &payload),
        ("exercise_templates", None) => client::create_exercise_template(&api_key, &payload),
        ("routine_folders", None) => client::create_routine_folder(&api_key, &payload),
        ("body_measurements", Some(date)) => {
            client::update_body_measurement(&api_key, &date, &payload)
        }
        ("body_measurements", None) => client::create_body_measurement(&api_key, &payload),
        _ => unreachable!("all mutation resources are known"),
    }
}

fn read_payload(source: &str) -> Result<serde_json::Value, AppError> {
    let content = if source == "-" {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content).map_err(|_| {
            AppError::invocation("Could not read JSON payload from standard input.")
        })?;
        content
    } else if let Some(path) = source.strip_prefix('@') {
        fs::read_to_string(path)
            .map_err(|_| AppError::invocation("Could not read the JSON payload file."))?
    } else {
        source.to_owned()
    };

    serde_json::from_str(&content)
        .map_err(|_| AppError::invocation("--data must contain valid JSON."))
}

fn validate_mutation_payload(
    resource_path: &str,
    is_update: bool,
    payload: &serde_json::Value,
) -> Result<(), AppError> {
    if resource_path != "routines" {
        return Ok(());
    }

    if !payload
        .get("routine")
        .is_some_and(serde_json::Value::is_object)
    {
        return Err(AppError::invocation(
            "Routine payload must contain a top-level \"routine\" object.",
        ));
    }

    if is_update {
        validate_routine_request::<PutRoutineRequest>("update", payload)
    } else {
        validate_routine_request::<PostRoutineRequest>("create", payload)
    }
}

fn validate_routine_request<T>(operation: &str, payload: &serde_json::Value) -> Result<(), AppError>
where
    T: for<'de> Deserialize<'de>,
{
    routine_request_validation_detail::<T>(payload).map_err(|detail| {
        AppError::invocation(format!("Invalid routine {operation} payload: {detail}."))
    })
}

fn routine_request_validation_detail<T>(payload: &serde_json::Value) -> Result<(), String>
where
    T: for<'de> Deserialize<'de>,
{
    serde_path_to_error::deserialize::<_, T>(payload)
        .map(|_| ())
        .map_err(|error| {
            let path = error.path().to_string();
            let message = error.inner().to_string();
            let message = message.split(" at line ").next().unwrap_or(&message);
            if message.starts_with("unknown field `") {
                format!("{path} is not accepted; omit response-only fields")
            } else {
                format!("{path}: {message}")
            }
        })
}

fn dry_run_output(
    resource_path: &str,
    resource_name: &str,
    resource_id: Option<&str>,
    payload: serde_json::Value,
) -> serde_json::Value {
    let (method, path, affected_resource) = match resource_id {
        Some(resource_id) => (
            "PUT",
            format!("/v1/{resource_path}/{resource_id}"),
            resource_id.to_owned(),
        ),
        None => (
            "POST",
            format!("/v1/{resource_path}"),
            format!("new {resource_name}"),
        ),
    };

    serde_json::json!({
        "dry_run": true,
        "affected_resource": affected_resource,
        "request": {
            "method": method,
            "path": path,
            "body": redact_secrets(payload),
        }
    })
}

fn redact_secrets(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(fields) => fields
            .into_iter()
            .map(|(name, value)| {
                let value = if is_secret_field(&name) {
                    serde_json::Value::String("[REDACTED]".to_owned())
                } else {
                    redact_secrets(value)
                };
                (name, value)
            })
            .collect(),
        serde_json::Value::Array(items) => items.into_iter().map(redact_secrets).collect(),
        value => value,
    }
}

fn is_secret_field(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "api_key" | "apikey" | "authorization" | "password" | "token" | "secret"
    )
}

fn exit_with(error: AppError, format: OutputFormat) -> ! {
    render::error(&error, format);
    process::exit(error.exit_code);
}

fn requests_json_output() -> bool {
    let arguments: Vec<_> = env::args().collect();
    arguments.iter().any(|argument| argument == "--format=json")
        || arguments
            .windows(2)
            .any(|arguments| arguments == ["--format", "json"])
}

fn parse_date(value: &str) -> Result<String, String> {
    let format = time::format_description::parse_borrowed::<3>("[year]-[month]-[day]")
        .expect("the date format description is valid");
    Date::parse(value, &format)
        .map(|_| value.to_owned())
        .map_err(|_| "must be a valid date in YYYY-MM-DD form, such as 2025-01-15".to_owned())
}

fn parse_iso8601(value: &str) -> Result<String, String> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map(|_| value.to_owned())
        .map_err(|_| {
            "must be an ISO-8601 timestamp with an offset, such as 2025-01-01T00:00:00Z".to_owned()
        })
}

fn resolve_api_key(explicit_key: Option<String>) -> Result<String, AppError> {
    explicit_key
        .filter(|key| !key.trim().is_empty())
        .or_else(|| env::var("HEVY_API_KEY").ok().filter(|key| !key.trim().is_empty()))
        .or_else(config_api_key)
        .ok_or_else(|| {
            AppError::authentication(
                "No API key was provided. Set HEVY_API_KEY, use --api-key, or configure hevy/config.toml.",
            )
        })
}

fn config_api_key() -> Option<String> {
    let path = config_directory()?.join("hevy").join("config.toml");
    let content = fs::read_to_string(path).ok()?;
    toml::from_str::<Config>(&content)
        .ok()?
        .api_key
        .filter(|key| !key.trim().is_empty())
}

fn config_directory() -> Option<std::path::PathBuf> {
    env::var_os("HEVY_CONFIG_DIR")
        .map(std::path::PathBuf::from)
        .or_else(dirs::config_dir)
}
