---
name: hevy-rs
description: Operate the hevy-rs CLI safely with JSON output, authenticated Hevy API access, and explicit mutation approval.
---

# hevy-rs

Use this skill to operate the `hevy-rs` CLI. Use `--format json` for data-bearing commands and invoke commands through the host's structured-argument mechanism, not dynamically interpolated shell text.

If `hevy-rs` is not available on `PATH` or the host cannot locate it, report that to the user and point them to the repository README's [Install on Linux](../../README.md#install-on-linux) section. Do not attempt an operation until the user has made the CLI available.

## Start here

- Read [common CLI guidance](references/common.md) before operating the CLI, especially before a mutation.
- Read the applicable resource reference for the command:
  - [Authenticated user](references/user.md)
  - [Workouts](references/workouts.md)
  - [Routines](references/routines.md)
  - [Exercise templates](references/exercise-templates.md)
  - [Routine folders](references/routine-folders.md)
  - [Exercise history](references/exercise-history.md)
  - [Body measurements](references/body-measurements.md)
- Use `hevy-rs --help` only when the installed CLI differs from this skill or a reference is not yet available.

Keep API keys out of prompts, logs, plans, and rendered output. Do not make a mutation without the exact, freshly approved plan required by the common guidance.
