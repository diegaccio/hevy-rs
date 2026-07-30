# hevy-rs

A Rust command-line interface for the [Hevy API](https://api.hevyapp.com/docs/).

> **Status:** Early development. The first supported command is authenticated user lookup.

## Build and first read

Use a stable Rust toolchain, then run:

```sh
cargo run -- --api-key "$HEVY_API_KEY" user get
```

For automation, store the key in `HEVY_API_KEY` and request the stable machine contract:

```sh
HEVY_API_KEY=... cargo run -- --format json user get
```

Credentials are resolved in this order: `--api-key`, `HEVY_API_KEY`, then the per-user
configuration file at `hevy/config.toml` below your platform's native configuration directory.
The configuration file contains `api_key = "..."`; on Linux, create its directory and file with
owner-only permissions (`chmod 700` for the directory and `chmod 600` for the file).

The CLI sends `GET /v1/user/info` with the API key in Hevy's `api-key` header. It never prints the
key in diagnostics. `--format json` writes errors to stderr and uses exit status 2 for invocation
errors, 3 for authentication errors, 4 for API errors, and 5 for transport or exhausted read retries.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE), or
- [MIT license](LICENSE-MIT)

at your option.
