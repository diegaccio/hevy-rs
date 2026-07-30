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
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

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
    /// Create a routine from a complete API-shaped JSON payload.
    Create(MutationArgs),
    /// Replace a routine with a complete API-shaped JSON payload.
    Update {
        /// Routine identifier.
        routine_id: String,
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

    /// Number of items per page (maximum 10 for workouts and routines).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=10))]
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
