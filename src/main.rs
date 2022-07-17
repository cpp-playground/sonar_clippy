use itertools::Itertools;
use std::fs;
use std::process::{Command, Stdio};

mod clippy;
mod sonar;

fn main() -> std::io::Result<()> {
    let mut command = Command::new("cargo")
        .args(&[
            "clippy",
            "--all-features",
            "--all-targets",
            "--no-deps",
            "--message-format=json",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let reader = std::io::BufReader::new(command.stdout.take().unwrap());

    let mut issues = sonar::IssuesList {
        issues: Vec::<sonar::Issue>::new(),
    };

    for message in cargo_metadata::Message::parse_stream(reader) {
        let message = message?;
        match sonar::Issue::try_from(message) {
            Ok(issue) => issues.issues.push(issue),
            Err(_) => println!("Error parsing message into issue"),
        };
    }

    issues.issues = issues.issues.into_iter().unique().collect();
    let j = serde_json::to_string(&issues)?;

    fs::write("sonar.json", j).expect("Unable to write file");

    Ok(())
}
