# hevy-rs

A Rust command-line interface for the [Hevy API](https://api.hevyapp.com/docs/).

> **Status:** Early development. Authenticated user lookup, workout commands, and routine management are supported.

## Build and first read

Use a stable Rust toolchain, then run:

```sh
cargo run -- --api-key "$HEVY_API_KEY" user get
```

Example output:

```text
ID: 00000000-0000-0000-0000-000000000000
Name: Ada Lovelace
URL: https://hevy.com/user/ada
```

For automation, store the key in `HEVY_API_KEY` and request the stable machine contract:

```sh
HEVY_API_KEY=... cargo run -- --format json user get
```

Example JSON output:

```json
{"id":"00000000-0000-0000-0000-000000000000","name":"Ada Lovelace","url":"https://hevy.com/user/ada"}
```

Credentials are resolved in this order: `--api-key`, `HEVY_API_KEY`, then the per-user
configuration file at `hevy/config.toml` below your platform's native configuration directory.
The configuration file contains `api_key = "..."`; on Linux, create its directory and file with
owner-only permissions (`chmod 700` for the directory and `chmod 600` for the file).

## Workouts

Use the `workouts` resource commands to inspect workout data:

```sh
hevy-rs workouts list
hevy-rs workouts count
hevy-rs workouts get <workout-id>
hevy-rs workouts events --since 2025-01-01T00:00:00Z
```

Example list output:

```text
Page: 1 of 1
- A - Lower strength + upper pull (0c00f518-81ea-4715-a68f-e12a6b2da836)
- Gym test (9e8d4c7c-9a32-452a-a69d-cf1b21c9ceb2)
```

Example count output:

```text
Workout count: 42
```

A workout lookup summarizes the workout and its exercises by default:

```text
ID: 0c00f518-81ea-4715-a68f-e12a6b2da836
Title: A - Lower strength + upper pull
Started: 2026-07-30T05:42:48+00:00
Ended: 2026-07-30T06:52:52+00:00
Exercises:
- Box Jump (3 sets)
- Squat (Barbell) (4 sets)
- Bent Over Row (Barbell) (4 sets)
```

Workout events identify updates and deletions:

```text
Page: 1 of 1
- Updated: A - Lower strength + upper pull (0c00f518-81ea-4715-a68f-e12a6b2da836)
- Deleted: 9e8d4c7c-9a32-452a-a69d-cf1b21c9ceb2
```

`list` and `events` accept `--page <n>` and `--page-size <n>` (1–10). Use `--all` to retrieve every page; it cannot be combined with `--page`. Collection JSON is normalized to `items`, `page`, and `page_count`; complete retrieval also reports `all` and `pages_fetched`.

Create or replace a workout with its complete API-shaped JSON body. `--data` accepts inline JSON, `@path` for a JSON file, or `-` for standard input:

Replace the exercise-template IDs with your existing Hevy template IDs. This example creates a workout with two exercises; use `--dry-run` first to review it without sending it:

```sh
cat <<'JSON' | hevy-rs workouts create --dry-run --data -
{
  "workout": {
    "title": "Upper body",
    "description": "",
    "start_time": "2026-07-30T17:00:00Z",
    "end_time": "2026-07-30T17:45:00Z",
    "is_private": false,
    "exercises": [
      {
        "exercise_template_id": "<bench-press-template-id>",
        "superset_id": null,
        "notes": "",
        "sets": [
          { "type": "normal", "weight_kg": 60, "reps": 8, "rpe": 8 }
        ]
      },
      {
        "exercise_template_id": "<barbell-row-template-id>",
        "superset_id": null,
        "notes": "",
        "sets": [
          { "type": "normal", "weight_kg": 50, "reps": 10, "rpe": 8 }
        ]
      }
    ]
  }
}
JSON
```

After reviewing the dry run, remove `--dry-run` to create the workout. Updates replace the complete workout body. For example, this update adds a second set to each existing exercise and adds Incline Bench Press (Dumbbell):

```sh
cat <<'JSON' | hevy-rs workouts update <workout-id> --data -
{
  "workout": {
    "title": "Hevy CLI Test Workout",
    "description": "",
    "start_time": "2026-07-30T17:00:00Z",
    "end_time": "2026-07-30T17:45:00Z",
    "is_private": false,
    "exercises": [
      {
        "exercise_template_id": "79D0BB3A",
        "superset_id": null,
        "notes": "",
        "sets": [
          { "type": "normal", "weight_kg": 60, "reps": 8, "rpe": 8 },
          { "type": "normal", "weight_kg": 60, "reps": 8, "rpe": 8 }
        ]
      },
      {
        "exercise_template_id": "55E6546F",
        "superset_id": null,
        "notes": "",
        "sets": [
          { "type": "normal", "weight_kg": 50, "reps": 10, "rpe": 8 },
          { "type": "normal", "weight_kg": 50, "reps": 10, "rpe": 8 }
        ]
      },
      {
        "exercise_template_id": "07B38369",
        "superset_id": null,
        "notes": "",
        "sets": [
          { "type": "normal", "weight_kg": 28, "reps": 8, "rpe": 8 }
        ]
      }
    ]
  }
}
JSON
```

Use your own workout and exercise-template IDs, and dry-run the exact payload before sending it.

## Routines

Use the `routines` resource commands to list and manage training plans:

```sh
hevy-rs routines list
hevy-rs routines list --all
hevy-rs routines get <routine-id>
hevy-rs routines create --data @routine.json
hevy-rs routines update <routine-id> --data @routine.json
```

`list` accepts `--page <n>` and `--page-size <n>` (1–10), or `--all` for explicit complete retrieval; `--all` cannot be combined with `--page`. In JSON mode, a collection is normalized to `items`, `page`, and `page_count`; `--all` also reports `all: true` and `pages_fetched`.

### Create a routine

A create payload is a complete object under `routine`. It needs a title, a destination `folder_id`, and the full list of exercises. Each exercise refers to an existing Hevy exercise-template ID and includes all of its planned sets. A set may use a fixed `reps` value or an inclusive `rep_range`.

Replace the IDs below with IDs from your account. The routine-folder ID is required when creating a routine; obtain it from Hevy before creating the payload.

```json
{
  "routine": {
    "title": "Test routine",
    "folder_id": 123456,
    "notes": "Created for API verification.",
    "exercises": [
      {
        "exercise_template_id": "<exercise-template-id-1>",
        "rest_seconds": 60,
        "notes": "",
        "sets": [
          { "type": "normal", "rep_range": { "start": 8, "end": 12 } }
        ]
      },
      {
        "exercise_template_id": "<exercise-template-id-2>",
        "rest_seconds": 60,
        "notes": "",
        "sets": [
          { "type": "normal", "reps": 10 }
        ]
      }
    ]
  }
}
```

Review the exact request first; this sends no API request:

```sh
hevy-rs --format json routines create --dry-run --data @routine.json
```

Then create it using the configured `HEVY_API_KEY`:

```sh
hevy-rs --format json routines create --data @routine.json
```

### Update a routine

An update replaces the complete routine body. Start by retrieving the routine, preserve every exercise and set you intend to keep, then send a complete payload. Unlike creation, an update payload does not include `folder_id`.

```sh
hevy-rs --format json routines get <routine-id>
hevy-rs --format json routines update <routine-id> --dry-run --data @updated-routine.json
hevy-rs --format json routines update <routine-id> --data @updated-routine.json
```

Routine reads and mutations return the Hevy API response, including the created or updated routine ID. Keep that ID to retrieve and reconcile the routine after a write.

Routine creates and updates accept `--data` as inline JSON, `@path`, or `-` for standard input. Use `--dry-run` to inspect the redacted request without sending it. A 4xx/5xx response reports a known API failure. Do not retry a routine mutation after a transport failure: its outcome is unknown, so retrieve and reconcile the affected routine first.

Use `--dry-run` with workout mutations to validate the JSON and inspect the redacted intended request without sending it. Do not repeat a workout mutation after a transport failure: its outcome is unknown; retrieve and reconcile the affected workout first.

The CLI sends `GET /v1/user/info` and documented workout and routine requests with the API key in Hevy's
`api-key` header. It never prints the key in diagnostics. `--format json` writes errors to stderr and uses exit status 2 for invocation
errors, 3 for authentication errors, 4 for API errors, and 5 for transport or exhausted read retries.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE), or
- [MIT license](LICENSE-MIT)

at your option.
