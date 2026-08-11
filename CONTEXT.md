# Hevy CLI

A Rust command-line interface for accessing the Hevy API. It is packaged with an agent skill that lets an agent operate the CLI reliably.

## Language

**`hevy-rs` skill**:
The bundled agent guide for operating the `hevy-rs` CLI efficiently and safely. Its name matches the CLI so agents can identify the tool it documents. It is distributed through the `npx skills add` ecosystem installer and provides a resource reference for every command the CLI supports.
_Avoid_: coaching workflow, generic Hevy skill

**Hevy data**:
Information in a user's Hevy account that the service exposes or manages, including profile details, workouts, routines, routine folders, exercise templates, exercise history, and body measurements.
_Avoid_: general information about Hevy or its API; the `hevy-rs` CLI itself

**Official Hevy API documentation**:
The authoritative source for API resource semantics and complete API-shaped JSON payload schemas. The `hevy-rs` skill links to it when an operation needs a payload definition.
_Avoid_: duplicated payload schema

**CLI-skill synchronization**:
The requirement that a change to the public `hevy-rs` CLI contract—commands, flags, defaults, output or error shapes, credential/configuration behavior, supported operations, or write/recovery semantics—updates the bundled `hevy-rs` skill in the same change. Internal behavior-preserving refactors are excluded.
_Avoid_: independently maintained CLI documentation
