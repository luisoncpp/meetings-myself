use super::error::ReportError;
use super::front_matter::ReportFrontMatter;

const DELIMITER: &str = "---";

#[derive(Debug)]
pub struct ReportDocument {
    pub front_matter: ReportFrontMatter,
    pub body: String,
}

impl ReportDocument {
    /// Consumes exactly the first two delimiters. A `---` horizontal rule later in
    /// the body is ordinary prose and must survive untouched.
    pub fn parse(text: &str) -> Result<Self, ReportError> {
        let rest = text
            .strip_prefix(DELIMITER)
            .and_then(|rest| rest.strip_prefix('\n'))
            .ok_or(ReportError::MissingFrontMatter)?;

        let (yaml, body) =
            split_at_closing_delimiter(rest).ok_or(ReportError::MissingFrontMatter)?;
        Ok(Self {
            front_matter: ReportFrontMatter::parse(yaml)?,
            body: body.to_string(),
        })
    }

    pub fn render(&self) -> Result<String, ReportError> {
        let yaml = self.front_matter.render()?;
        Ok(format!("{DELIMITER}\n{yaml}{DELIMITER}\n{}", self.body))
    }
}

/// Finds the closing `---` that stands alone on its own line.
fn split_at_closing_delimiter(rest: &str) -> Option<(&str, &str)> {
    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == DELIMITER {
            let body_start = offset + line.len();
            return Some((&rest[..offset], &rest[body_start..]));
        }
        offset += line.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReportError;
    use chrono::NaiveDate;

    const SAMPLE: &str = "---\n\
week: 2026-W32\n\
week_start: 2026-08-03\n\
week_end: 2026-08-09\n\
schema: 1\n\
generated_at: 2026-08-09T18:22:11Z\n\
---\n\
\n\
## Reflection\n\
\n\
It was a good week.\n";

    #[test]
    fn a_document_splits_into_front_matter_and_body() {
        let document = ReportDocument::parse(SAMPLE).unwrap();
        assert_eq!(document.front_matter.week, "2026-W32");
        assert_eq!(
            document.front_matter.week_start,
            NaiveDate::from_ymd_opt(2026, 8, 3).unwrap()
        );
        assert_eq!(document.front_matter.schema, 1);
        assert_eq!(document.body, "\n## Reflection\n\nIt was a good week.\n");
    }

    #[test]
    fn rendering_a_parsed_document_reproduces_it_byte_for_byte() {
        let document = ReportDocument::parse(SAMPLE).unwrap();
        assert_eq!(document.render().unwrap(), SAMPLE);
    }

    #[test]
    fn a_body_containing_a_horizontal_rule_is_not_mistaken_for_a_delimiter() {
        let text = format!("{SAMPLE}\n---\n\nMore prose after a rule.\n");
        let document = ReportDocument::parse(&text).unwrap();
        assert!(document.body.contains("More prose after a rule."));
        assert_eq!(document.front_matter.week, "2026-W32");
    }

    #[test]
    fn a_file_without_front_matter_is_reported_not_guessed_at() {
        assert!(matches!(
            ReportDocument::parse("Just some notes.\n").unwrap_err(),
            ReportError::MissingFrontMatter
        ));
    }

    #[test]
    fn an_unknown_schema_is_refused_rather_than_silently_overwritten() {
        let text = SAMPLE.replace("schema: 1", "schema: 99");
        assert!(matches!(
            ReportDocument::parse(&text).unwrap_err(),
            ReportError::UnsupportedSchema { found: 99 }
        ));
    }
}
