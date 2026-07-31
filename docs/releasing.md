# Release policy

`hevy-rs` is released as a source package to crates.io. Initial releases support Linux
through `cargo install hevy-rs`; they do not include prebuilt binaries or other GitHub
Release assets.

## One-time repository setup

1. Configure crates.io trusted publishing for this repository and the `crates.io`
   GitHub Actions environment. The publisher must be this repository's `Release`
   workflow; do not add a long-lived crates.io token as a repository secret.
2. Create a GitHub ruleset or protected-tag rule for `v*` that prevents unapproved
   creation, update, and deletion of release tags. Limit tag creation to maintainers.
3. Protect the `crates.io` environment with required reviewers and restrict deployment
   branches/tags to protected `v*` tags. This environment approval is the final gate
   before the OIDC publishing credential is minted.

These GitHub and crates.io settings cannot be represented solely in the repository, so
release automation deliberately names the protected environment and documents the
required controls here.

## Release procedure

1. Review the dependency update and `Cargo.lock`. Run the local checks:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo deny check
   cargo package --locked
   ```

2. Update `Cargo.toml` to the intended stable SemVer version and merge it through the
   normal review process. Stable means exactly `MAJOR.MINOR.PATCH`: no prerelease or
   build suffix is eligible for publication.
3. Create the protected tag `vMAJOR.MINOR.PATCH` at the reviewed release commit. The
   tag must exactly match the package version; the workflow rejects any mismatch.
4. Approve the `crates.io` environment deployment. The workflow repeats all quality
   and dependency checks, packages with the committed lockfile, obtains a short-lived
   crates.io credential through GitHub OIDC, publishes, and creates the GitHub Release.

The GitHub Release notes record the commit and successful checks as CI evidence. They
contain no binary assets.

## Dependency policy and exceptions

`deny.toml` is the release gate for RustSec advisories, yanked packages, licenses, and
package sources. The policy allows only crates.io registry packages and a small list of
widely compatible licenses. It denies known advisories and yanked versions.

Do not silence a finding in CI. If a temporary exception is unavoidable, add the narrow
exception to `deny.toml` with an adjacent comment identifying the affected package,
reason, tracking issue, and removal version or date. Review and remove exceptions at
every release. A dependency source or license outside the policy requires the same
explicit, reviewed exception; prefer replacing the dependency instead.
