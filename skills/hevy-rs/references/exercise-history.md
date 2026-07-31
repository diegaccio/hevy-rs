# Exercise history

Use the exercise-template identifier exactly as returned in an `id` field by `exercise-templates list`, `exercise-templates get`, or an exercise template created by the CLI. Identifiers are opaque strings; do not derive or alter them.

Use [common CLI guidance](common.md) for credentials, structured argument handling, and JSON results.

## Retrieve history

Retrieve one exercise template's history with explicit inclusive bounds when the requested period is known. `--start` includes entries on or after the supplied timestamp; `--end` includes entries on or before it. Each bound must be an ISO-8601 timestamp with an offset, for example `2025-01-01T00:00:00Z`.

```sh
hevy-rs --format json exercise-history get <exercise-template-id> --start 2025-01-01T00:00:00Z --end 2025-01-31T23:59:59Z
```

Omit either bound only when that open-ended range is intentional.

For resource semantics and response shapes, see the official [get-exercise-history operation](https://api.hevyapp.com/docs/#/ExerciseHistory/get_v1_exercise_history__exerciseTemplateId_).
