---
name: hevy
summary: Safely use the hevy-rs CLI to inspect Hevy training data, provide bounded non-medical coaching support, and perform explicitly approved writes.
---

# Hevy coaching workflow

Use this workflow when assisting a person with their Hevy training data. It is runtime-neutral: adapt the invocation mechanism to the host, but preserve the rules below.

## Safety boundary

- Provide general, non-medical training support only. Do not diagnose, treat, prescribe, or replace qualified care.
- Stop the coaching workflow and encourage appropriate qualified professional support for injury or pain, pregnancy, eating-disorder context, medication questions, or any other material health risk.
- Do not infer goals, constraints, experience, available equipment, schedule, or relevant health context from Hevy records. Ask for missing information before recommending a change.
- Treat Hevy data as incomplete unless the requested resource and time window have been explicitly retrieved and validated.

## Invocation and result handling

1. For an unfamiliar CLI version or command, run its help before acting, for example `hevy-rs --help` or `hevy-rs workouts --help`.
2. Invoke every data-bearing command as a structured argument array, never by constructing a shell command through string interpolation. For example:

   ```text
   ["hevy-rs", "--format", "json", "workouts", "get", "<workout-id>"]
   ```

3. Capture process exit status, stdout, and stderr separately. Do not treat stderr as data or parse presentation-oriented output.
4. Pass `--format json` for every data-bearing command. Parse stdout as JSON only after a successful exit status.
5. Validate the expected JSON shape before using it. Verify required object fields and types; for collections, validate `items`, `page`, and `page_count` before treating the result as a collection.
6. Stop and clarify rather than guessing if a command fails, stderr contains a JSON error, stdout is malformed, required fields are absent, pagination is partial when completeness matters, or the returned data conflicts with the user's account of events.
7. Never place API keys in prompts, logs, plans, or rendered output. Prefer the user's existing credential configuration.

## Gather bounded evidence

Before advice, establish the person's goals and constraints. Ask about the objective, experience, schedule, equipment, preferences, recovery, and any other information necessary for the requested recommendation. Respect the safety boundary above.

Retrieve only the records necessary to answer the question, and state the evidence boundary. Use explicit time bounds for exercise history when relevant:

```text
["hevy-rs", "--format", "json", "exercise-history", "get", "<exercise-template-id>", "--start", "2026-01-01T00:00:00Z", "--end", "2026-01-31T23:59:59Z"]
```

For paginated resources, retrieve the specific pages needed. Use `--all` only when complete retrieval is necessary, and record that choice and the pages returned. Do not combine `--all` with `--page`.

## Present advice traceably

Before proposing a write, present a bounded, non-medical recommendation containing all of the following:

- the assessment based on the retrieved records and the user's stated goals and constraints;
- the proposed training change;
- uncertainty, missing information, and assumptions;
- each source resource identifier used (for example workout, routine, exercise-template, or body-measurement IDs); and
- the exact time window covered by the evidence, or an explicit statement that no time-bounded evidence was used.

Do not present a recommendation as certain when the evidence is insufficient. Ask a clarifying question or recommend observation instead.

## Plan and approve writes

An agent-initiated write is never authorized merely because the CLI permits it without a prompt. Before any create or update:

1. Build one exact, finite write plan. Name every operation, its target, and its complete API-shaped JSON payload. Include the execution order if it matters.
2. Use `--dry-run` with the same target and payload where available to validate the intended request without sending it. Inspect its result and redact secrets.
3. Present the entire batch to the person. Do not hide, summarize away, add, reorder, or substitute payloads after approval.
4. Obtain fresh, explicit approval for that exact batch. Approval must identify the plan or unambiguously confirm every listed operation. Silence, prior approval, broad permission, and approval of a similar plan are not approval.
5. Execute only the approved operations, once each. A changed payload, target, operation, order, added operation, or later batch requires a new plan and fresh explicit approval.

Use structured data rather than shell interpolation for write input. For example, provide the complete JSON via a controlled file or standard input and invoke:

```text
["hevy-rs", "--format", "json", "routines", "update", "<routine-id>", "--data", "@approved-routine.json"]
```

Record the execution result for each operation separately. A successful process and valid response establish a known result; an API error establishes a known failure. Do not claim either result without validating it.

## Outcome-unknown mutations

A transport failure after a mutation request can leave its outcome unknown. The CLI intentionally does not retry mutations automatically. If this occurs:

1. Stop the batch immediately. Do not retry, compensate for, or continue dependent writes.
2. Tell the person which operation has an unknown outcome and why.
3. Read the relevant resource or collection and reconcile the intended change against the returned data. Use a bounded, validated JSON read.
4. Present the reconciliation result, including remaining uncertainty if the result is still ambiguous.
5. Obtain fresh explicit approval before repeating the mutation, issuing a compensating mutation, or resuming with any new write batch.

Never use a previous approval to resolve an outcome-unknown mutation.
