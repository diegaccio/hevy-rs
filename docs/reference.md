# hevy-rs reference

This reference describes the public command contract. Run `hevy-rs --help` for the
contract of the installed version; the Hevy public API is early and may change.

## Installation and command discovery

Linux users need a current stable Rust toolchain. From a source checkout, install with:

```sh
cargo install --path .
hevy-rs --help
```

The command grammar is resource-first. Inspect help before using an unfamiliar command:

```sh
hevy-rs --help
hevy-rs workouts --help
hevy-rs workouts update --help
```

## Credentials and configuration

The CLI resolves credentials, skipping blank values, in this strict order:

1. `--api-key <KEY>`
2. `HEVY_API_KEY`
3. `hevy/config.toml` under the platform-native per-user configuration directory

The configuration file contains:

```toml
api_key = "your-api-key"
```

On Linux, create the configuration directory with mode `700` and the file with mode
`600`, for example:

```sh
install -d -m 700 "${XDG_CONFIG_HOME:-$HOME/.config}/hevy"
install -m 600 /dev/stdin "${XDG_CONFIG_HOME:-$HOME/.config}/hevy/config.toml" <<'EOF'
api_key = "your-api-key"
EOF
```

`HEVY_CONFIG_DIR` overrides the configuration base directory, which is useful for
isolated automation and tests. Do not put a key in a project-local configuration file,
commit it, or place it in a shell command that will be retained in history. The CLI
redacts recognized secret fields from dry-run output and does not print the API key in
its diagnostics.

## Output and errors

Readable text is the default. `--format json` is the machine interface: successful
JSON is written to stdout, and an error object is written to stderr. The JSON error
object always has `code` and `message`, and may include `status`, `request_id`, and
`retry_after_seconds`.

| Exit | `code` | Meaning |
| ---: | --- | --- |
| 2 | `invocation` | Invalid command, argument, or input |
| 3 | `authentication` | Missing or rejected API key |
| 4 | `api` | Known API/HTTP failure |
| 5 | `transport` | Transport failure or exhausted read retry/rate-limit budget |

Single resources and workout counts preserve the API object. Collection results are
normalized without changing individual resources:

```json
{"items":[...],"page":1,"page_count":3}
```

With `--all`, collections also contain `"all": true` and `pages_fetched`, recording
that complete retrieval was requested and which pages were read.

## Commands

All listed operations map to documented public Hevy API operations.

| Resource | Commands |
| --- | --- |
| Authenticated user | `user get` |
| Workouts | `workouts list`, `count`, `get <workout-id>`, `events`, `create`, `update <workout-id>` |
| Routines | `routines list`, `get <routine-id>`, `create`, `update <routine-id>` |
| Exercise templates | `exercise-templates list`, `get <exercise-template-id>`, `create` |
| Routine folders | `routine-folders list`, `get <folder-id>`, `create` |
| Exercise history | `exercise-history get <exercise-template-id>` |
| Body measurements | `body-measurements list`, `get <YYYY-MM-DD>`, `create`, `update <YYYY-MM-DD>` |

Examples:

```sh
hevy-rs user get
hevy-rs workouts count
hevy-rs workouts get <workout-id>
hevy-rs routines get <routine-id>
hevy-rs exercise-templates get <exercise-template-id>
hevy-rs routine-folders get <folder-id>
hevy-rs body-measurements get 2025-01-15
hevy-rs exercise-history get <exercise-template-id> \
  --start 2025-01-01T00:00:00Z --end 2025-01-31T23:59:59Z
```

`workouts events` accepts `--since <ISO-8601>`. Exercise-history `--start` and `--end`
are ISO-8601 timestamps with an offset, such as `2025-01-01T00:00:00Z`. A body
measurement identifier must be a real `YYYY-MM-DD` date.

### Pagination

These commands accept `--page <n>`, `--page-size <n>`, and `--all`:

- `workouts list` and `workouts events`
- `routines list`
- `routine-folders list`
- `body-measurements list`
- `exercise-templates list`

Page numbers start at 1. Page sizes are 1–10 except exercise templates, which permit
1–100. Omit `--page-size` to retain the API default. `--all` reads every page explicitly
and cannot be combined with `--page`.

```sh
hevy-rs workouts list --page 2 --page-size 10
hevy-rs exercise-templates list --page-size 100
hevy-rs routines list --all
```

### Creates and updates

Creates and updates require one complete API-shaped JSON body:

```text
--data <JSON|@path|->
```

The value may be inline JSON, `@path` to a JSON file, or `-` to read JSON from standard
input. Nested API bodies are preserved; there is no per-field flag grammar.

Routine creates and updates require a top-level `routine` object, for example
`{"routine":{"title":"Upper","exercises":[]}}`. `routines get` also returns a top-level
`routine` object, but its nested resource includes response-only fields. To update it, build a
new request from its mutable fields. `folder_id` is mutable (use `null` for the default folder); omit
routine `id`, `created_at`, and `updated_at`, exercise `index` and `title`, and set `index`.

```sh
hevy-rs routines create --data @routine.json
printf '%s' '{"routine_folder":{"title":"Strength"}}' |
  hevy-rs routine-folders create --data -
hevy-rs body-measurements update 2025-01-15 --data @replacement.json
```

Use `--dry-run` on every mutation to parse the body and show the redacted intended HTTP
method, path, body, and affected resource without sending a request. Routine creates and updates
also validate the complete operation-specific request schema locally before previewing; an invalid
field produces an invocation error with its JSON path. `--dry-run` and
`--yes` cannot be combined. Current public commands contain no destructive operation,
so no command requires `--yes`; the flag is reserved for a future documented irreversible
operation. Reversible creates and updates do not prompt.

Body-measurement creation conflicts when that date already exists. Its update replaces
the complete entry: an omitted measurement becomes `null`. Read the existing entry first
and include every value to retain.

## Safety and recovery

Read requests retry conservatively after a pre-response transport failure, HTTP 429, or
transient 5xx response. The CLI makes at most three retries after the first request,
using full-jitter exponential backoff from 500 ms; it honors a valid `Retry-After` no
larger than 60 seconds.

Mutations are never retried automatically. If a create or update reports a transport
failure, the server may have accepted it even though no response arrived. Do not repeat,
compensate for, or continue dependent writes. Read the affected resource or collection,
reconcile the intended change, then obtain fresh approval before any follow-up write.

## Unavailable operations

The CLI deliberately does **not** offer undocumented public-API operations. In
particular, deletion is unavailable. There is also no routine-folder update/delete,
exercise-template update/delete, or account mutation command. Do not simulate these
operations or assume that a missing command has an equivalent workaround.

## Fixture testing

The test suite uses a deterministic local HTTP fixture server rather than live Hevy
credentials. It invokes the compiled CLI, observes its exit status, stdout, stderr, and
HTTP request, and verifies request methods, paths, queries, redacted authentication
handling, and complete JSON bodies. Fixture responses exercise success, local validation,
pagination, dry runs, error mapping, retries, and mutations whose outcomes are unknown.

Run the checks from a checkout:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
