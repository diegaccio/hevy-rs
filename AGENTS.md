# hevy-rs

This repository contains a Rust command-line interface for the Hevy API.

API documentation: <https://api.hevyapp.com/docs/>

## Project direction

- Keep the application as a Rust CLI binary.
- Build API interactions around the Hevy API.
- Prefer clear, idiomatic Rust and add tests as functionality is introduced.

## Live API verification

- When a fact relevant to a change could be verified more reliably against the live Hevy API than against documentation alone, ask the user for permission to use the `HEVY_API_KEY` environment variable. Do not assume that permission from its presence.
- Once permission is granted, use the live API to validate the fact rather than relying blindly on published API documentation. Prefer a read-only request whenever it can answer the question.
- If a live mutation is necessary, obtain explicit approval for that mutation and target only the dedicated test resources: routine `[TEST] hevy-rs API verification (update check)` (`498f951f-db4e-46ea-b72f-6e525d7a9ff9`) or routine folder `Test` (`3331054`). Never modify a non-test resource for API verification.
- The Hevy API has no documented delete operation. Preserve the test resources and avoid creating additional verification data unless the user explicitly approves it.

## CLI-skill synchronization

A public CLI-contract change covers commands and subcommands, flags and defaults, output or error shapes, credential or configuration behavior, supported API operations, and write or recovery semantics. Internal behavior-preserving refactors are excluded.

For every covered change, update the bundled `skills/hevy-rs` skill in the same change. If the skill needs no update, explicitly record `Skill impact: none — <reason>` in the issue or pull request describing the change.

## Agent skills

### Issue tracker

Issues are tracked in this repository's GitHub Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Uses the default five canonical triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

Uses a single-context domain-doc layout. See `docs/agents/domain.md`.
