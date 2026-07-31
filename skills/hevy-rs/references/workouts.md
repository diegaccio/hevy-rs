# Workouts

Use workout identifiers exactly as returned in an `id` field by `workouts list`, `workouts get`, or an updated workout event. A deleted event supplies the affected workout identifier in its `id` field. Identifiers are opaque strings; do not derive or alter them.

Use [common CLI guidance](common.md) for credentials, structured argument handling, JSON results, and the shared mutation protocol.

## Read and synchronize

List one bounded page of workouts. `--page` starts at 1 and `--page-size` is at most 10; use `--all` only when every page is required (it cannot be combined with `--page`).

```sh
hevy-rs --format json workouts list --page 1 --page-size 10
```

Count all workouts:

```sh
hevy-rs --format json workouts count
```

Retrieve one workout's complete details:

```sh
hevy-rs --format json workouts get <workout-id>
```

Retrieve workout change events since an ISO-8601 timestamp with an offset. Events are newest first and indicate updated or deleted workouts. `--since` bounds the event stream; combine it with the same bounded pagination options as `workouts list` (`--page` starts at 1, `--page-size` is at most 10, or `--all` without `--page`).

```sh
hevy-rs --format json workouts events --since 2025-01-01T00:00:00Z --page 1 --page-size 10
```

For API semantics and response shapes, see the official operations for [listing workouts](https://api.hevyapp.com/docs/#/Workouts/get_v1_workouts), [counting workouts](https://api.hevyapp.com/docs/#/Workouts/get_v1_workouts_count), [retrieving a workout](https://api.hevyapp.com/docs/#/Workouts/get_v1_workouts__workoutId_), and [retrieving workout events](https://api.hevyapp.com/docs/#/Workouts/get_v1_workouts_events).

## Create and update

Follow the shared mutation protocol in [common CLI guidance](common.md): form an exact finite plan, inspect a `--dry-run`, obtain fresh explicit approval, execute the approved batch once, and reconcile before any follow-up after an outcome-unknown mutation. Supply the complete API-shaped JSON payload with `--data <JSON|@path|->`; use a controlled file or standard input for dynamic payloads. Do not copy a complete payload schema into this skill.

Create a workout:

```sh
hevy-rs --format json workouts create --dry-run --data @workout.json
```

After approval, rerun the same command without `--dry-run`.

Replace an existing workout with its complete payload:

```sh
hevy-rs --format json workouts update <workout-id> --dry-run --data @workout.json
```

After approval, rerun the same command without `--dry-run`.

Use the official [create-workout operation](https://api.hevyapp.com/docs/#/Workouts/post_v1_workouts) and [update-workout operation](https://api.hevyapp.com/docs/#/Workouts/put_v1_workouts__workoutId_) for the current complete payload schemas and operation semantics.
