# Routines

Use routine identifiers exactly as returned in an `id` field by `routines list`, `routines get`, or a routine created or updated by the CLI. Identifiers are opaque strings; do not derive or alter them.

Use [common CLI guidance](common.md) for credentials, structured argument handling, JSON results, and the shared mutation protocol.

## List and retrieve

List one bounded page of routines. `--page` starts at 1 and `--page-size` is at most 10; use `--all` only when every page is required (it cannot be combined with `--page`).

```sh
hevy-rs --format json routines list --page 1 --page-size 10
```

Retrieve one routine's complete details:

```sh
hevy-rs --format json routines get <routine-id>
```

For API semantics and response shapes, see the official operations for [listing routines](https://api.hevyapp.com/docs/#/Routines/get_v1_routines) and [retrieving a routine](https://api.hevyapp.com/docs/#/Routines/get_v1_routines__routineId_).

## Create and update

Follow the shared mutation protocol in [common CLI guidance](common.md): form an exact finite plan, inspect a `--dry-run`, obtain fresh explicit approval, execute the approved batch once, and reconcile before any follow-up after an outcome-unknown mutation. Supply the complete API-shaped JSON payload with `--data <JSON|@path|->`; use a controlled file or standard input for dynamic payloads. Do not copy a complete payload schema into this skill.

Create a routine:

```sh
hevy-rs --format json routines create --dry-run --data @routine.json
```

After approval, rerun the same command without `--dry-run`.

Replace an existing routine with its complete payload:

```sh
hevy-rs --format json routines update <routine-id> --dry-run --data @routine.json
```

After approval, rerun the same command without `--dry-run`.

Use the official [create-routine operation](https://api.hevyapp.com/docs/#/Routines/post_v1_routines) and [update-routine operation](https://api.hevyapp.com/docs/#/Routines/put_v1_routines__routineId_) for the current complete payload schemas and operation semantics.
