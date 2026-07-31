# Hevy public API and CLI landscape

Research for [Research the Hevy API and competing CLIs](https://github.com/diegaccio/hevy-rs/issues/2). Sources were checked 2025-12-30.

## Public API contract

Hevy describes its public API as an early, Pro-only offering, available with a key from Developer Settings, and explicitly reserves the right to change or abandon the API. Every documented endpoint accepts the `api-key` request header. [Official API documentation](https://api.hevyapp.com/docs/)

| Resource | Public operations |
| --- | --- |
| Workouts | list (`GET /v1/workouts`), count, change events, get, create, replace/update (`PUT`) |
| User | get authenticated user information |
| Routines | list, get, create, replace/update (`PUT`) |
| Exercise templates | list, get, create custom template |
| Routine folders | list, get, create |
| Exercise history | get by exercise-template ID, optional ISO-8601 start/end range |
| Body measurements | list, get by `YYYY-MM-DD`, create, replace/update (`PUT`) |

Source: [official API documentation](https://api.hevyapp.com/docs/). The same currently documented operation inventory is captured in the [generated OpenAPI snapshot](https://github.com/tifandotme/hevy-cli/blob/master/docs/hevy-openapi.json), whose update script downloads it from `https://api.hevyapp.com/docs/openapi.json` ([script](https://github.com/tifandotme/hevy-cli/blob/master/scripts/update-openapi.ts)).

### Constraints and semantics that affect the CLI

- Page-based collection endpoints use `page` (default 1; must be >= 1) and `pageSize`. The maximum is **10** for workouts, workout events, routines, folders, and body measurements; it is **100** for exercise templates. The documented defaults are normally 5, except body measurements (10).
- Workout events are newest-first, can be filtered by `since` (default epoch), and exist to synchronize a local workout cache. Events distinguish updated workouts from deleted workouts.
- Body-measurement creation returns conflict when its date already exists. Its `PUT` is replacement semantics: omitted measurement fields become `null`.
- Routine-folder creation inserts at index 0 and increments the indices of existing folders.
- Workout/routine write payloads contain nested exercises and sets. Sets support `warmup`, `normal`, `failure`, and `dropset`; workout writes support an RPE enum from 6 through 10 in 0.5 increments. Units in documented fields are kg, metres, seconds, and ISO-8601 timestamps.
- Public API coverage is **not** full account control: it exposes no documented DELETE operation, no routine-folder update/delete, no exercise-template update/delete, and no user/account mutation. The CLI must accurately surface this limitation rather than invent unsupported commands.

Sources: [official API documentation](https://api.hevyapp.com/docs/); [OpenAPI operation and schema snapshot](https://github.com/tifandotme/hevy-cli/blob/master/docs/hevy-openapi.json).

## Comparable CLIs

### `obay/hevycli`

Strong points: broad command surface for workouts, routines, exercises, and folders; shell completion; table/JSON/plain output; documented error exit codes; native binaries and interactive flows. It also derives analytics such as progress and records from API data, which is useful but is application logic rather than API parity.

Gaps/caveats: its advertised CRUD includes deletes and folder updates even though those methods are absent from the documented public API, so that claim needs independent verification before copying it. Its environment key is `HEVYCLI_API_KEY`, not the ecosystem-familiar `HEVY_API_KEY`; its config example stores a plaintext key under `~/.hevycli`.

Source: [`obay/hevycli` README](https://github.com/obay/hevycli#readme).

### `tifandotme/hevy-cli`

Strong points: intentionally API-shaped command groups; `HEVY_API_KEY` overrides local configuration; explicit `--all` pagination; JSON request bodies accepted inline or from files; generated types from an OpenAPI snapshot; tests around API, auth, body handling, output, and smoke behavior; an installable agent skill.

Gaps/caveats: its README calls it an early release, defaults to compact JSON rather than offering a human-oriented terminal default, and it currently presents only a subset of the API in its examples. Its checked-in OpenAPI snapshot is a useful drift detector but must not substitute for checking the official docs at release time.

Sources: [`tifandotme/hevy-cli` README](https://github.com/tifandotme/hevy-cli#readme), [agent skill](https://github.com/tifandotme/hevy-cli/blob/master/skills/hevy-cli/SKILL.md), and [OpenAPI update script](https://github.com/tifandotme/hevy-cli/blob/master/scripts/update-openapi.ts).

### `marinsalinas/hevy-cli`

Strong points: declares all public resource groups, auto-pagination, typed API models, retries for rate limits/server errors, structured debug logs, TTY-sensitive table/JSON/YAML output, XDG configuration, and mock-based tests. It is the clearest precedent for separating human-friendly presentation from machine output.

Gaps/caveats: it offers an API-key command-line flag, which can expose a secret through process listings/history; its agent skill is Claude-specific; and its coverage table does not include user info or body measurements, both of which the public API documents.

Sources: [`marinsalinas/hevy-cli` README](https://github.com/marinsalinas/hevy-cli#readme) and [CLI source](https://github.com/marinsalinas/hevy-cli/tree/main/src/hevy_cli).

### `Dor256/hevy-cli`

Strong points: small Go binary with simple login/setup, GitHub releases, CI, and an embedded agent skill. It focuses on a narrow, understandable workflow: view workouts and manage routines.

Gaps/caveats: its public README names only workouts and routines, omitting user info, exercise templates/history, folders, and body measurements; it does not document output modes, pagination behavior, error contract, or non-interactive mutation safeguards.

Source: [`Dor256/hevy-cli` README](https://github.com/Dor256/hevy-cli#readme).

## Implications for the specification

1. Treat the official API as the parity boundary: every documented operation needs a command, while unsupported desired operations need an explicit API-gap response, not faux support.
2. Normalize pagination behind a consistent command contract, with `--all` and API-appropriate page-size caps rather than silently assuming a shared cap.
3. Make stable JSON an explicit output mode for agents; preserve readable terminal output for people. Model write bodies as JSON/file inputs to keep the complete nested API shape available without a sprawling flag grammar.
4. Adopt `HEVY_API_KEY` plus local config with environment precedence. Do not require a secret-bearing CLI flag; if it is ever included, document its exposure risk.
5. Guard every supported mutation with the map's confirmation policy. There are no documented destructive mutation endpoints today, but create/replace operations still change account data.
6. Include retry/error/exit-code behavior and API-schema drift checking in later decisions; Hevy's stated API instability makes them part of a reliable client contract.
