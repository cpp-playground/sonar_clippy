use crate::sonar;
use cargo_metadata::diagnostic::{Diagnostic, DiagnosticLevel, DiagnosticSpan};
use cargo_metadata::Message;

impl From<DiagnosticLevel> for sonar::Severity {
    fn from(value: DiagnosticLevel) -> sonar::Severity {
        match value {
            DiagnosticLevel::Ice => sonar::Severity::Blocker,
            DiagnosticLevel::Error => sonar::Severity::Critical,
            DiagnosticLevel::FailureNote => sonar::Severity::Major,
            DiagnosticLevel::Warning => sonar::Severity::Minor,
            _ => sonar::Severity::Info,
        }
    }
}

impl From<&DiagnosticSpan> for sonar::TextRange {
    fn from(value: &DiagnosticSpan) -> sonar::TextRange {
        sonar::TextRange {
            start_line: value.line_start,
            end_line: value.line_end,
            start_column: Some(value.column_start - 1),
            end_column: Some(value.column_end - 1),
        }
    }
}

fn extract_locations(
    diagnostic: &Diagnostic,
) -> Result<(sonar::Location, Option<Vec<sonar::Location>>), ()> {
    if let Some(primary) = diagnostic.spans.iter().find(|span| span.is_primary) {
        let privary_location = sonar::Location {
            file_path: primary.file_name.clone(),
            message: diagnostic.message.clone(),
            text_range: Some(sonar::TextRange::from(primary)),
        };

        let secondary_locations: Vec<sonar::Location> = diagnostic
            .spans
            .iter()
            .filter(|span| !span.is_primary)
            .map(|span| sonar::Location {
                file_path: span.file_name.clone(),
                message: diagnostic.message.clone(),
                text_range: Some(sonar::TextRange::from(span)),
            })
            .collect();

        return Ok((privary_location, Some(secondary_locations)));
    }

    Err(())
}

impl TryFrom<Diagnostic> for sonar::Issue {
    type Error = ();
    fn try_from(diagnostic: Diagnostic) -> Result<Self, Self::Error> {
        let (primary, secondary) = extract_locations(&diagnostic)?;
        Ok(sonar::Issue {
            engine_id: "clippy".to_string(),
            primary_location: primary,
            rule_id: diagnostic.code.as_ref().map_or_else(
                || String::from("clippy"),
                |diagnostic_code| diagnostic_code.code.clone(),
            ),
            severity: sonar::Severity::from(diagnostic.level),
            secondary_locations: secondary,
            r#type: sonar::Type::CodeSmell,
        })
    }
}

impl TryFrom<cargo_metadata::Message> for sonar::Issue {
    type Error = ();
    fn try_from(value: cargo_metadata::Message) -> Result<Self, Self::Error> {
        if let Message::CompilerMessage(msg) = value {
            sonar::Issue::try_from(msg.message)
        } else {
            Err(())
        }
    }
}
