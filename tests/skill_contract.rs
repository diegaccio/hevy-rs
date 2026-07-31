use assert_cmd::Command;
use std::{
    collections::BTreeSet,
    fs,
    process::{Command as ProcessCommand, Output},
};

const SKILL_REFERENCES: &str = "skills/hevy-rs/references";

#[test]
fn every_supported_command_has_a_canonical_invocation_in_its_skill_reference() {
    let binary = Command::cargo_bin("hevy-rs").unwrap();
    let commands = discover_commands(binary.get_program());

    assert!(!commands.is_empty(), "the CLI help tree has no commands");

    for command in commands {
        let reference = skill_reference_for(&command);
        let reference_contents = fs::read_to_string(&reference)
            .unwrap_or_else(|error| panic!("could not read {}: {error}", reference.display()));
        let invocation = format!("hevy-rs --format json {}", command.join(" "));

        assert!(
            reference_contents.contains(&invocation),
            "{} is missing its canonical invocation: {invocation}",
            reference.display()
        );
    }
}

fn discover_commands(binary: &std::ffi::OsStr) -> BTreeSet<Vec<String>> {
    let mut pending = vec![Vec::new()];
    let mut commands = BTreeSet::new();

    while let Some(path) = pending.pop() {
        let output = run_help(binary, &path);
        let subcommands = subcommands_in(&output);

        if subcommands.is_empty() {
            commands.insert(path);
        } else {
            pending.extend(subcommands.into_iter().map(|subcommand| {
                let mut child = path.clone();
                child.push(subcommand);
                child
            }));
        }
    }

    commands
}

fn run_help(binary: &std::ffi::OsStr, path: &[String]) -> Output {
    ProcessCommand::new(binary)
        .args(path)
        .arg("--help")
        .output()
        .unwrap_or_else(|error| panic!("could not run help for `{}`: {error}", path.join(" ")))
}

fn subcommands_in(output: &Output) -> Vec<String> {
    assert!(
        output.status.success(),
        "help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let help = String::from_utf8_lossy(&output.stdout);
    let Some((_, commands)) = help.split_once("Commands:\n") else {
        return Vec::new();
    };

    commands
        .lines()
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .filter(|command| *command != "help")
        .map(str::to_owned)
        .collect()
}

fn skill_reference_for(command: &[String]) -> std::path::PathBuf {
    let resource = command
        .first()
        .expect("only commands beneath the CLI root are documented");
    let filename = match resource.as_str() {
        "user" => "user.md",
        "workouts" => "workouts.md",
        "routines" => "routines.md",
        "exercise-templates" => "exercise-templates.md",
        "routine-folders" => "routine-folders.md",
        "exercise-history" => "exercise-history.md",
        "body-measurements" => "body-measurements.md",
        _ => panic!("no skill reference is assigned to `{resource}`"),
    };

    std::path::Path::new(SKILL_REFERENCES).join(filename)
}
