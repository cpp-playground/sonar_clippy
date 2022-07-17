use cargo_metadata::Message;
use std::{
    fs::{self, File},
    io::Write,
    process::{Command, Stdio},
};

use serde::Serialize;

#[derive(Serialize)]
struct IssuesList {
    issues: Vec<Issue>,
}

#[derive(Serialize)]
struct TextRange {
    startLine: i32,
    endLine: i32,
    startColumn: Option<i32>,
    endColumn: Option<i32>,
}

#[derive(Serialize)]
struct Location {
    message: String,
    filePath: String,
    textRange: Option<TextRange>,
}

#[derive(Serialize)]
struct Issue {
    engineId: String,
    ruleId: String,
    primaryLocation: Location,
    severity: String,
    r#type: String,
}

fn main() -> std::io::Result<()> {
    let mut issues = IssuesList {
        issues: Vec::<Issue>::new(),
    };

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

    for message in cargo_metadata::Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerMessage(msg) => {
                if !msg.message.spans.is_empty() {
                    let primary_text_range = TextRange {
                        startLine: msg.message.spans[0].line_start as i32,
                        endLine: msg.message.spans[0].line_end as i32,
                        startColumn: Some((msg.message.spans[0].column_start - 1) as i32),
                        endColumn: Some((msg.message.spans[0].column_end - 1) as i32),
                    };
                    let primary_location = Location {
                        message: msg.message.message,
                        filePath: msg.message.spans[0].file_name.clone(),
                        textRange: Some(primary_text_range),
                    };

                    let issue = Issue {
                        engineId: "clippy".to_string(),
                        ruleId: msg.message.code.as_ref().map_or_else(
                            || String::from("clippy"),
                            |diagnostic_code| diagnostic_code.code.clone(),
                        ),
                        primaryLocation: primary_location,
                        severity: "MINOR".to_string(),
                        r#type: "CODE_SMELL".to_string(),
                    };

                    issues.issues.push(issue);
                }
            }
            _ => (), // Unknown message
        }
    }

    let j = serde_json::to_string(&issues)?;

    fs::write("sonar.json", j).expect("Unable to write file");

    Ok(())
}
