# Body measurements

A body measurement is addressed by its `date`, not an opaque identifier. Pass a valid calendar date in `YYYY-MM-DD` form exactly as returned by `body-measurements list`, `body-measurements get`, or a created measurement. For example, `2025-01-15` is valid; `2025-02-29` is not.

Use [common CLI guidance](common.md) for credentials, structured argument handling, JSON results, and the shared mutation protocol.

## List and retrieve

List one bounded page of body measurements. `--page` starts at 1 and `--page-size` is at most 10; use `--all` only when every page is required (it cannot be combined with `--page`).

```sh
hevy-rs --format json body-measurements list --page 1 --page-size 10
```

Retrieve one measurement by date:

```sh
hevy-rs --format json body-measurements get 2025-01-15
```

For API semantics and response shapes, see the official operations for [listing body measurements](https://api.hevyapp.com/docs/#/Measurements/get_v1_body_measurements) and [retrieving a body measurement](https://api.hevyapp.com/docs/#/Measurements/get_v1_body_measurements__date_).

## Create and replace

Follow the shared mutation protocol in [common CLI guidance](common.md): form an exact finite plan, inspect a `--dry-run`, obtain fresh explicit approval, execute the approved batch once, and reconcile before any follow-up after an outcome-unknown mutation. Supply the complete API-shaped JSON payload with `--data <JSON|@path|->`; use a controlled file or standard input for dynamic payloads. Do not copy a complete payload schema into this skill.

Create a measurement:

```sh
hevy-rs --format json body-measurements create --dry-run --data @body-measurement.json
```

After approval, rerun the same command without `--dry-run`.

Before replacing a measurement, first retrieve the existing value and use it to prepare the complete replacement payload. An update overwrites all fields, so omitted fields are set to `null`.

```sh
hevy-rs --format json body-measurements update 2025-01-15 --dry-run --data @body-measurement.json
```

After approval, rerun the same command without `--dry-run`.

Use the official [create-body-measurement operation](https://api.hevyapp.com/docs/#/Measurements/post_v1_body_measurements) and [update-body-measurement operation](https://api.hevyapp.com/docs/#/Measurements/put_v1_body_measurements__date_) for the current complete payload schemas and operation semantics.
