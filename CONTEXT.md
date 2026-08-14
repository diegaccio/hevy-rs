# Hevy CLI

A Rust command-line interface for accessing the Hevy API. It is packaged with an agent skill that lets an agent operate the CLI reliably.

## Language

**`hevy-rs` skill**:
The bundled agent guide for operating the `hevy-rs` CLI efficiently and safely. Its name matches the CLI so agents can identify the tool it documents. It is distributed through the `npx skills add` ecosystem installer and provides a resource reference for every command the CLI supports.
_Avoid_: coaching workflow, generic Hevy skill

**Hevy data**:
Information in a user's Hevy account that the service exposes or manages, including profile details, workouts, routines, routine folders, exercise templates, exercise history, and body measurements.
_Avoid_: general information about Hevy or its API; the `hevy-rs` CLI itself

**AI coach**:
A user-selected AI agent that provides fitness guidance and uses the `hevy-rs` CLI and skill to access approved Hevy data. It is external to `hevy-rs`; the project is a data-access tool, not a coaching product.
_Avoid_: `hevy-rs` personal trainer, autonomous coach

**Official Hevy API documentation**:
The authoritative source for API resource semantics and complete API-shaped JSON payload schemas. The `hevy-rs` skill links to it when an operation needs a payload definition.
_Avoid_: duplicated payload schema

**CLI-skill synchronization**:
The requirement that a change to the public `hevy-rs` CLI contract—commands, flags, defaults, output or error shapes, credential/configuration behavior, supported operations, or write/recovery semantics—updates the bundled `hevy-rs` skill in the same change. Internal behavior-preserving refactors are excluded.
_Avoid_: independently maintained CLI documentation

**Routine resource**:
The nested `routine` object returned by a routine retrieval operation. It contains response-only fields and is not, unchanged, a routine mutation body.
_Avoid_: routine update payload, routine request

**Routine request envelope**:
A top-level JSON object containing a `routine` object, required as the body for a routine create or update operation. A routine retrieval response uses the same outer envelope but its nested resource has different allowed fields.
_Avoid_: unwrapped routine, routine resource
