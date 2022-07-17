use serde::Serialize;

#[derive(Serialize)]
pub struct IssuesList {
    pub issues: Vec<Issue>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub engine_id: String,
    pub rule_id: String,
    pub primary_location: Location,
    pub severity: Severity,
    pub r#type: Type,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_locations: Option<Vec<Location>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub message: String,
    pub file_path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_range: Option<TextRange>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    pub start_line: usize,
    pub end_line: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_column: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    Bug,
    Vulnerability,
    CodeSmell,
}

#[derive(Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Blocker,
    Critical,
    Major,
    Minor,
    Info,
}
