# hevy-rs

This repository contains a Rust command-line interface for the Hevy API.

API documentation: <https://api.hevyapp.com/docs/>

## Project direction

- Keep the application as a Rust CLI binary.
- Build API interactions around the Hevy API.
- Prefer clear, idiomatic Rust and add tests as functionality is introduced.

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
