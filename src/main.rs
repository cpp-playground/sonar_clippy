use cargo_metadata::Message;
use serde::Serialize;
use std::fs;
use std::process::{Command, Stdio};

#[derive(Serialize)]
struct IssuesList {
    issues: Vec<Issue>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TextRange {
    start_line: i32,
    end_line: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_column: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_column: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Location {
    message: String,
    file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text_range: Option<TextRange>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Issue {
    engine_id: String,
    rule_id: String,
    primary_location: Location,
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
        if let Message::CompilerMessage(msg) = message.unwrap() {
            if !msg.message.spans.is_empty() {
                let primary_text_range = TextRange {
                    start_line: msg.message.spans[0].line_start as i32,
                    end_line: msg.message.spans[0].line_end as i32,
                    start_column: Some((msg.message.spans[0].column_start - 1) as i32),
                    end_column: Some((msg.message.spans[0].column_end - 1) as i32),
                };
                let primary_location = Location {
                    message: msg.message.message,
                    file_path: msg.message.spans[0].file_name.clone(),
                    text_range: Some(primary_text_range),
                };

                let issue = Issue {
                    engine_id: "clippy".to_string(),
                    rule_id: msg.message.code.as_ref().map_or_else(
                        || String::from("clippy"),
                        |diagnostic_code| diagnostic_code.code.clone(),
                    ),
                    primary_location,
                    severity: "MINOR".to_string(),
                    r#type: "CODE_SMELL".to_string(),
                };

                issues.issues.push(issue);
            }
        }
    }

    let j = serde_json::to_string(&issues)?;

    fs::write("sonar.json", j).expect("Unable to write file");

    Ok(())
}
