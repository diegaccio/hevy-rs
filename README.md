# hevy-rs

A Rust command-line interface for the [Hevy API](https://api.hevyapp.com/docs/).

> **Status:** Early development. The CLI supports the public authenticated-user, workout,
> routine, exercise-template, routine-folder, exercise-history, and body-measurement operations.

## Install

Install [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) with
[rustup](https://rustup.rs/), clone this repository, then install the binary:

```sh
git clone https://github.com/diegaccio/hevy-rs.git
cd hevy-rs
cargo install --path .
hevy-rs --help
```

To run directly from a checkout instead, replace `hevy-rs` below with `cargo run --`.

## First safe read

Create an API key in Hevy, keep it out of shell history, and supply it through the
environment:

```sh
export HEVY_API_KEY='your-api-key'
hevy-rs user get
```

Use JSON for scripts and agents:

```sh
hevy-rs --format json user get
```

Discover the resource-first command tree before using a new operation:

```sh
hevy-rs --help
hevy-rs workouts --help
hevy-rs workouts create --help
```

## Agent skill

Install the bundled `hevy-rs` skill with the ecosystem installer:

```sh
npx skills add diegaccio/hevy-rs --skill hevy-rs
```

The skill is an operating guide for the CLI; see [its entry guide](skills/hevy-rs/SKILL.md).

## Documentation

- [Command, configuration, safety, recovery, and fixture reference](docs/reference.md)
- [Human command-discovery walkthrough](docs/walkthroughs/discovery.md)
- [Evidence-to-advice walkthrough](docs/walkthroughs/evidence-to-advice.md)
- [Proposal-to-write walkthrough](docs/walkthroughs/proposal-to-write.md)
- [Ambiguous-write recovery walkthrough](docs/walkthroughs/ambiguous-write-recovery.md)
- [hevy-rs agent skill](skills/hevy-rs/SKILL.md)
- [Release policy and maintainer procedure](docs/releasing.md)

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE), or
- [MIT license](LICENSE-MIT)

at your option.
