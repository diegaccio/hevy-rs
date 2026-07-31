---
name: hevy-rs
description: Operate the hevy-rs CLI safely with JSON output, authenticated Hevy API access, and explicit mutation approval.
---

# hevy-rs

Use this skill to operate the `hevy-rs` CLI. Use `--format json` for data-bearing commands and invoke commands through the host's structured-argument mechanism, not dynamically interpolated shell text.

## Start here

- Read [common CLI guidance](references/common.md) before operating the CLI, especially before a mutation.
- Read the applicable resource reference for the command:
  - [Authenticated user](references/user.md)
  - [Workouts](references/workouts.md)
  - Routines, exercise templates, routine folders, exercise history, and body measurements: their focused references are added alongside those resource commands.
- Use `hevy-rs --help` only when the installed CLI differs from this skill or a reference is not yet available.

Keep API keys out of prompts, logs, plans, and rendered output. Do not make a mutation without the exact, freshly approved plan required by the common guidance.
