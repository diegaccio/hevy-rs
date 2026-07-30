# hevy-rs

A Rust command-line interface for the [Hevy API](https://api.hevyapp.com/docs/).

> **Status:** Early development. Authenticated user lookup and workout retrieval are supported.

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

After reviewing the dry run, remove `--dry-run` to create the workout. To update an existing workout, use `hevy-rs workouts update <workout-id> --data @workout.json`.

Use `--dry-run` with either mutation to validate the JSON and inspect its redacted intended request without sending it. Do not repeat a mutation after a transport failure: its outcome is unknown; retrieve and reconcile the affected workout first.

The CLI sends `GET /v1/user/info` and documented workout requests with the API key in Hevy's
`api-key` header. It never prints the key in diagnostics. `--format json` writes errors to stderr and uses exit status 2 for invocation
errors, 3 for authentication errors, 4 for API errors, and 5 for transport or exhausted read retries.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE), or
- [MIT license](LICENSE-MIT)

at your option.
