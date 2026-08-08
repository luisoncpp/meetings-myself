use super::error::ReportError;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// App-owned metadata. Written in snake_case because a human edits this file by
/// hand — unlike the IPC types, which are camelCase for TypeScript.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportFrontMatter {
    pub week: String,
    pub week_start: NaiveDate,
    pub week_end: NaiveDate,
    pub schema: u32,
    pub generated_at: DateTime<Utc>,
}

impl ReportFrontMatter {
    pub const SCHEMA: u32 = 1;

    pub fn render(&self) -> Result<String, ReportError> {
        serde_norway::to_string(self)
            .map_err(|error| ReportError::MalformedFrontMatter { detail: error.to_string() })
    }

    pub fn parse(yaml: &str) -> Result<Self, ReportError> {
        let parsed: Self = serde_norway::from_str(yaml)
            .map_err(|error| ReportError::MalformedFrontMatter { detail: error.to_string() })?;
        if parsed.schema > Self::SCHEMA {
            return Err(ReportError::UnsupportedSchema { found: parsed.schema });
        }
        Ok(parsed)
    }
}
