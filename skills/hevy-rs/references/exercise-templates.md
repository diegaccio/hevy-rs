# Exercise templates

Use exercise-template identifiers exactly as returned in an `id` field by `exercise-templates list`, `exercise-templates get`, or an exercise template created by the CLI. Identifiers are opaque strings; do not derive or alter them.

Use [common CLI guidance](common.md) for credentials, structured argument handling, JSON results, and the shared mutation protocol.

## List and retrieve

List one bounded page of exercise templates. `--page` starts at 1 and `--page-size` is at most 100; use `--all` only when every page is required (it cannot be combined with `--page`). This expanded 100-item limit applies only to `exercise-templates list`; other paginated resource commands have their own limits.

```sh
hevy-rs --format json exercise-templates list --page 1 --page-size 100
```

Retrieve one exercise template:

```sh
hevy-rs --format json exercise-templates get <exercise-template-id>
```

For API semantics and response shapes, see the official operations for [listing exercise templates](https://api.hevyapp.com/docs/#/ExerciseTemplates/get_v1_exercise_templates) and [retrieving an exercise template](https://api.hevyapp.com/docs/#/ExerciseTemplates/get_v1_exercise_templates__exerciseTemplateId_).

## Create

Follow the shared mutation protocol in [common CLI guidance](common.md): form an exact finite plan, inspect a `--dry-run`, obtain fresh explicit approval, execute the approved batch once, and reconcile before any follow-up after an outcome-unknown mutation. Supply this documented input shape with `--data <JSON|@path|->`; use a controlled file or standard input for dynamic payloads:

```json
{
  "title": "Banded Lat Stretch",
  "type": "reps_only",
  "equipment": "resistance_band",
  "primary_muscle_group": "lats",
  "secondary_muscle_groups": ["shoulders"]
}
```

Create an exercise template:

```sh
hevy-rs --format json exercise-templates create --dry-run --data @exercise-template.json
```

The dry run displays the production wire payload, which nests `exercise` and maps the field names. After approval, rerun the same command without `--dry-run`. The JSON result is either the API response object or, when the API returns its normal plain-text response, a JSON string containing the created template ID.

Use the official [create-exercise-template operation](https://api.hevyapp.com/docs/#/ExerciseTemplates/post_v1_exercise_templates) for the current complete payload schema and operation semantics.
