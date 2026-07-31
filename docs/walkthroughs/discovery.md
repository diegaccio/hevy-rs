# Walkthrough: discover a safe read

This walkthrough verifies the installed command grammar and account connection without
writing data. It requires an API key but makes no mutation.

```sh
export HEVY_API_KEY='your-api-key'
hevy-rs --help
hevy-rs workouts --help
hevy-rs --format json user get
```

Capture the last command's exit status, stdout, and stderr separately. On success,
stdout is one JSON user object; do not parse default text output in automation.

Next, discover a bounded collection request:

```sh
hevy-rs workouts list --help
hevy-rs --format json workouts list --page 1 --page-size 5
```

Confirm that stdout has `items`, `page`, and `page_count`. If you need complete data,
make that choice explicit rather than assuming one page is sufficient:

```sh
hevy-rs --format json workouts list --all
```

Do not combine `--all` with `--page`. If a command is unfamiliar, return to `--help`
before using it.
