mod client;
mod error;
mod render;

use clap::{Args, Parser, Subcommand};
use error::AppError;
use render::OutputFormat;
use serde::Deserialize;
use std::{env, fs, process};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Parser)]
#[command(
    name = "hevy-rs",
    version,
    about = "A command-line client for the Hevy API"
)]
struct Cli {
    /// API key. Prefer HEVY_API_KEY or the per-user configuration file in automation.
    #[arg(long, env = "HEVY_API_KEY", global = true)]
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
}

#[derive(Subcommand)]
enum UserCommand {
    /// Retrieve the authenticated user's information.
    Get,
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
}

#[derive(Args)]
struct PaginationArgs {
    /// Page number, starting at 1.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..), conflicts_with = "all")]
    page: Option<u32>,

    /// Number of items per page (maximum 10 for workouts).
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
    let result = resolve_api_key(cli.api_key).and_then(|api_key| match cli.command {
        Command::User {
            command: UserCommand::Get,
        } => client::get_user(&api_key),
        Command::Workouts { command } => match command {
            WorkoutCommand::List(pagination) => client::list_workouts(
                &api_key,
                client::Pagination {
                    page: pagination.page,
                    page_size: pagination.page_size,
                    all: pagination.all,
                },
            ),
            WorkoutCommand::Count => client::get_workout_count(&api_key),
            WorkoutCommand::Get { workout_id } => client::get_workout(&api_key, &workout_id),
            WorkoutCommand::Events { pagination, since } => client::list_workout_events(
                &api_key,
                client::Pagination {
                    page: pagination.page,
                    page_size: pagination.page_size,
                    all: pagination.all,
                },
                since.as_deref(),
            ),
        },
    });

    match result {
        Ok(user) => render::success(&user, format),
        Err(error) => exit_with(error, format),
    }
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
