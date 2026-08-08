use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("could not read or write the report file")]
    Io(#[from] std::io::Error),

    #[error("the file has no YAML front matter block")]
    MissingFrontMatter,

    #[error("the front matter is not valid: {detail}")]
    MalformedFrontMatter { detail: String },

    #[error("this report uses schema {found}, which this version does not understand")]
    UnsupportedSchema { found: u32 },
}
