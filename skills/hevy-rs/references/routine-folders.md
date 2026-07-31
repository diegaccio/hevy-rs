# Routine folders

Use a routine-folder identifier exactly as returned in an `id` field by `routine-folders list`, `routine-folders get`, or a routine folder created by the CLI. An identifier can be numeric; pass its rendered value unchanged and do not derive or alter it.

Use [common CLI guidance](common.md) for credentials, structured argument handling, JSON results, and the shared mutation protocol.

## List and retrieve

List one bounded page of routine folders. `--page` starts at 1 and `--page-size` is at most 10; use `--all` only when every page is required (it cannot be combined with `--page`).

```sh
hevy-rs --format json routine-folders list --page 1 --page-size 10
```

Retrieve one routine folder:

```sh
hevy-rs --format json routine-folders get <folder-id>
```

For API semantics and response shapes, see the official operations for [listing routine folders](https://api.hevyapp.com/docs/#/RoutineFolders/get_v1_routine_folders) and [retrieving a routine folder](https://api.hevyapp.com/docs/#/RoutineFolders/get_v1_routine_folders__folderId_).

## Create

Follow the shared mutation protocol in [common CLI guidance](common.md): form an exact finite plan, inspect a `--dry-run`, obtain fresh explicit approval, execute the approved batch once, and reconcile before any follow-up after an outcome-unknown mutation. Supply the complete API-shaped JSON payload with `--data <JSON|@path|->`; use a controlled file or standard input for dynamic payloads. Do not copy a complete payload schema into this skill.

Create a routine folder:

```sh
hevy-rs --format json routine-folders create --dry-run --data @routine-folder.json
```

After approval, rerun the same command without `--dry-run`.

Use the official [create-routine-folder operation](https://api.hevyapp.com/docs/#/RoutineFolders/post_v1_routine_folders) for the current complete payload schema and operation semantics.
