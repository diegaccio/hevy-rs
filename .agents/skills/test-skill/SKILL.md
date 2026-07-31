---
name: test-skill
description: Tests the local hevy-rs CLI and its bundled hevy-rs operating skill against a live read-only Hevy account. Use when validating that documented read commands, JSON output, identifiers, and resource references remain synchronized.
compatibility: Requires a local hevy-rs repository checkout, Cargo, network access, and HEVY_API_KEY.
disable-model-invocation: true
---

# Test the local hevy-rs skill

Use this skill only to test the local repository's CLI and `skills/hevy-rs` operating guide. Do not run any create, update, or other mutation command.

## Preflight

1. Check that `HEVY_API_KEY` is set and non-blank without displaying its value. For example:

   ```sh
   test -n "${HEVY_API_KEY:-}"
   ```

   If it is absent or blank, tell the user: **`HEVY_API_KEY` is not set; the live read-only CLI and skill check was not run.** Stop. Do not fall back to `--api-key` or a configuration file: this test specifically verifies the environment-variable credential path.
2. Read [`skills/hevy-rs/SKILL.md`](../../../skills/hevy-rs/SKILL.md), then its common guidance and every resource reference it links. Report each missing linked reference as a skill failure, but continue with the applicable live CLI checks where possible.
3. From the repository root, use this local CLI command prefix in every invocation:

   ```sh
   cargo run --quiet -- --format json
   ```

   Treat the shell forms below as command grammar only. Invoke them with the host's structured-argument mechanism; never interpolate IDs, dates, or other dynamic values into shell text.

## What to verify

For every command below:

- Expect a zero exit status and valid JSON on stdout.
- Treat any stderr output, non-zero exit, invalid JSON, documented command mismatch, or unexpected response shape as a CLI or skill failure. Include the command name, exit status, and redacted error/output summary in the report; never expose the API key.
- Use only bounded list requests. Do not use `--all`.
- After each successful list, retain only the JSON identifiers needed for dependent reads. Identifiers are opaque; do not transform them.
- If a dependent resource does not exist, mark its command **skipped — no account data**, not failed. State that the check is incomplete.

### Credential and account

```sh
cargo run --quiet -- --format json user get
```

Confirm that the result is a JSON object and that the command is documented by the authenticated-user reference.

### Workouts

```sh
cargo run --quiet -- --format json workouts list --page 1 --page-size 10
cargo run --quiet -- --format json workouts count
cargo run --quiet -- --format json workouts events --page 1 --page-size 10
```

Confirm list and events return the documented normalized collection shape: `items`, `page`, and `page_count`. From the first workout item's `id`, when present, run:

```sh
cargo run --quiet -- --format json workouts get <workout-id>
```

### Routines

```sh
cargo run --quiet -- --format json routines list --page 1 --page-size 10
```

Confirm the normalized collection shape. From the first item's `id`, when present, run:

```sh
cargo run --quiet -- --format json routines get <routine-id>
```

### Exercise templates and history

```sh
cargo run --quiet -- --format json exercise-templates list --page 1 --page-size 10
```

Confirm the normalized collection shape. From the first item's `id`, when present, run both:

```sh
cargo run --quiet -- --format json exercise-templates get <exercise-template-id>
cargo run --quiet -- --format json exercise-history get <exercise-template-id>
```

### Routine folders

```sh
cargo run --quiet -- --format json routine-folders list --page 1 --page-size 10
```

Confirm the normalized collection shape. From the first item's `id`, when present, run:

```sh
cargo run --quiet -- --format json routine-folders get <folder-id>
```

### Body measurements

```sh
cargo run --quiet -- --format json body-measurements list --page 1 --page-size 10
```

Confirm the normalized collection shape. From the first item's `date`, when present, run:

```sh
cargo run --quiet -- --format json body-measurements get <date>
```

## Skill-contract review

Compare the CLI's current read-only help tree with the `hevy-rs` skill and references you read:

```sh
cargo run --quiet -- --help
cargo run --quiet -- user --help
cargo run --quiet -- workouts --help
cargo run --quiet -- routines --help
cargo run --quiet -- exercise-templates --help
cargo run --quiet -- routine-folders --help
cargo run --quiet -- exercise-history --help
cargo run --quiet -- body-measurements --help
```

Report a **skill failure** when a public read-only command lacks a linked focused reference or a canonical JSON-oriented invocation, or when documented identifiers, pagination limits, flags, defaults, JSON/error behavior, or write-safety language disagree with help or observed behavior. Do not claim a failure for an unavailable dependent identifier; report it as incomplete coverage instead.

## Final report

Give the user a concise report with:

- `PASS`, `FAIL`, or `INCOMPLETE` for each read-only command.
- Separate **CLI failures** from **skill failures**.
- Every skipped dependent read and why it was skipped.
- A final verdict: `PASS` only if every executable check passed and the skill review found no failure; otherwise `FAIL` for any failure, or `INCOMPLETE` when there are only skips.
- The exact next action needed to resolve each failure or incomplete check.
