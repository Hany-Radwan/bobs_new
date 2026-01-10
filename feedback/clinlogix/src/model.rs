use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub enum Severity {
    #[default]
    #[serde(rename = "information")]
    Information,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "fatal")]
    Fatal,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub severity: Severity,
    pub code: String,
    pub diagnostics: String,
    pub details: Option<Details>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Details {
    pub text: String,
}
