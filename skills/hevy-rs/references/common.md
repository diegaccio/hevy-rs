# Common CLI guidance

## Credentials

The CLI resolves the first non-blank credential in this order:

1. `--api-key <KEY>`
2. `HEVY_API_KEY`
3. `hevy/config.toml` in the platform-native per-user configuration directory

Prefer `HEVY_API_KEY` or the per-user configuration file. Do not put an API key in a prompt, log, plan, committed file, or a shell command retained in history. `HEVY_CONFIG_DIR` overrides the configuration base directory for isolated automation.

## Invocation and results

Use concise shell examples as grammar references, but execute them through the host's safe structured-argument mechanism. Never construct a command by interpolating dynamic values into shell text.

```sh
hevy-rs --format json user get
```

For data-bearing commands, use `--format json`. On success, parse JSON only from stdout after a zero exit status. On failure, treat stdout as non-data and parse the JSON error from stderr:

| Exit | Error code | Meaning |
| ---: | --- | --- |
| 2 | `invocation` | Invalid command, argument, or input |
| 3 | `authentication` | Missing or rejected API key |
| 4 | `api` | Known API/HTTP failure |
| 5 | `transport` | Transport failure or exhausted read retry/rate-limit budget |

Every error object has `code` and `message`; it can also contain `status`, `request_id`, and `retry_after_seconds`.

## Pagination

Paginated commands accept `--page <n>`, `--page-size <n>`, and `--all`. Page numbers start at 1; `--all` cannot be combined with `--page`. Retrieve a bounded page by default. Use `--all` only when complete retrieval is required, and verify the returned `items`, `page`, and `page_count`. An `--all` response also records `all: true` and `pages_fetched`.

## Mutation data and approval

Creates and updates accept a complete API-shaped JSON payload through:

```text
--data <JSON|@path|->
```

Use inline JSON only when it is safely supplied as a single structured argument; otherwise use a controlled file (`@path`) or standard input (`-`). Do not duplicate complete payload schemas here: use the official Hevy API operation linked by the applicable resource reference.

Before every write batch:

1. Form one exact, finite plan naming each operation, target, complete payload, and required order.
2. Run the same command with `--dry-run` where available; inspect its redacted intended request without sending it.
3. Present the entire plan and obtain fresh, explicit approval for that exact batch.
4. Execute each approved operation once only. Any changed target, payload, order, or added operation requires a new plan and approval.

`--dry-run` and `--yes` cannot be combined. `--yes` is reserved for a future documented irreversible operation; current creates and updates do not require it.

## Outcome-unknown mutations

Mutations are never retried automatically. If a mutation reports a transport failure, stop the batch: the server might have accepted it. Do not retry, compensate, or continue dependent writes. Read the affected resource or collection, reconcile it with the intended change, report the result and any remaining uncertainty, then obtain fresh explicit approval before any follow-up write.
