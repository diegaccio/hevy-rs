# Walkthrough: recover after an ambiguous write

A mutation transport failure is not evidence that the server rejected the request. The
request may have been accepted after the connection failed, so its result is unknown.

Suppose this approved routine update returns exit status 5 or a transport error:

```sh
hevy-rs --format json routines update <routine-id> --data @approved-routine.json
```

Follow this recovery sequence.

1. Stop the batch. Do not retry the update, issue a compensating write, or continue a
   dependent operation.
2. Tell the person the exact operation and target with the unknown outcome, and why it
   is unknown.
3. Read and validate the relevant data:

   ```sh
   hevy-rs --format json routines get <routine-id>
   ```

   If a create's ID is unknown, use a bounded relevant collection read and compare the
   complete intended payload with returned records.
4. Reconcile the intended change against the validated response. Report whether it is
   present, absent, or still ambiguous, including remaining uncertainty.
5. Before repeating the mutation, compensating for it, or resuming any write batch,
   build a new exact plan and obtain fresh explicit approval.

Never reuse the approval that authorized the outcome-unknown request. The CLI
intentionally never retries mutations automatically for this reason.
