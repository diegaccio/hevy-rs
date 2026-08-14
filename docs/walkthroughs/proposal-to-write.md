# Walkthrough: proposal to approved write

This walkthrough creates or updates only after a person approves one exact finite batch.
The example changes a routine; replace placeholders with real values.

1. Retrieve the current routine so the proposal preserves every exercise and set that
   should remain:

   ```sh
   hevy-rs --format json routines get <routine-id>
   ```

2. Write the complete replacement **request body** to a controlled file, for example
   `approved-routine.json`. Do not construct JSON through shell interpolation. The GET result
   already has a top-level `routine` object, but its nested resource is not an update body:
   omit `id`, `folder_id`, `created_at`, and `updated_at`, and retain the mutable fields needed
   for the replacement, for example `{"routine":{"title":"Upper","exercises":[]}}`.
3. Validate the exact target and body without sending a request:

   ```sh
   hevy-rs --format json routines update <routine-id> \
     --dry-run --data @approved-routine.json
   ```

4. Present the whole plan to the person: operation (`routines update`), target routine
   ID, complete API-shaped JSON payload, and execution order. Include the dry-run result.
5. Obtain fresh explicit approval that unambiguously covers this exact plan. Silence,
   broad permission, earlier approval, or approval of a similar plan is not approval.
6. Execute the approved operation once:

   ```sh
   hevy-rs --format json routines update <routine-id> \
     --data @approved-routine.json
   ```

7. Record the exit status, stdout, and stderr. A successful exit with valid JSON is a
   known result; an API error is a known failure.

A changed payload, target, operation, order, extra operation, or later batch requires a
new plan and fresh approval. The CLI does not prompt for this reversible write; that does
not waive the agent's approval requirement.
