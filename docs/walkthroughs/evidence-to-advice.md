# Walkthrough: bounded evidence to non-medical advice

This is an agent-safe workflow for helping a person interpret training records. It does
not write to Hevy.

1. Ask for the person's goal, experience, schedule, equipment, preferences, recovery,
   and constraints. Do not infer them from Hevy records.
2. Stop and encourage qualified professional support rather than coaching through injury
   or pain, pregnancy, eating-disorder context, medication questions, or another material
   health risk.
3. Choose the smallest evidence window that can answer the question. For example, to
   inspect January performance for an existing exercise template:

   ```sh
   hevy-rs --format json exercise-history get <exercise-template-id> \
     --start 2026-01-01T00:00:00Z --end 2026-01-31T23:59:59Z
   ```

4. Capture stdout, stderr, and exit status separately. Only after a successful exit,
   validate the expected JSON fields and types. Stop and clarify if data is missing,
   malformed, partial, contradictory, or the window is insufficient.
5. Present a non-medical recommendation that names:
   - the assessment;
   - the proposed training change;
   - uncertainty, assumptions, and missing information;
   - every source resource ID; and
   - the exact evidence window (`2026-01-01T00:00:00Z` through
     `2026-01-31T23:59:59Z` in this example).

For paginated evidence, request the needed pages or `--all` when completeness matters,
and report that choice and the returned pages. A recommendation with incomplete evidence
must say so; observation or a clarifying question is preferable to false certainty.
