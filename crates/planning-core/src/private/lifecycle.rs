use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Whether an entity is in everyday use or resting in the Archive. Archiving is
/// always reversible and never deletes (ADR 0002).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Lifecycle {
    #[default]
    Active,
    Archived,
}

impl Lifecycle {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

/// A Task's outcome. Orthogonal to `Lifecycle`: archiving a completed Task keeps
/// the completion, so restoring it returns it exactly as it was.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Completion {
    #[default]
    Open,
    Completed {
        on: NaiveDate,
    },
}

impl Completion {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }
}

/// A Goal's outcome. Same orthogonality as `Completion`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Achievement {
    #[default]
    Pursuing,
    Achieved {
        on: NaiveDate,
    },
}

impl Achievement {
    pub fn is_achieved(&self) -> bool {
        matches!(self, Self::Achieved { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 8, 7).unwrap()
    }

    #[test]
    fn lifecycle_and_completion_are_independent_axes() {
        // An archived Task remembers that it was completed, so restoring it does
        // not have to guess. This is why they are not one enum.
        let completed = Completion::Completed { on: date() };
        assert!(completed.is_complete());
        assert!(!Lifecycle::Archived.is_active());
        assert!(Lifecycle::Active.is_active());
    }

    #[test]
    fn serde_tags_are_camel_case_for_the_frontend_mirror() {
        assert_eq!(
            serde_json::to_string(&Lifecycle::Archived).unwrap(),
            r#""archived""#
        );
        assert_eq!(
            serde_json::to_string(&Completion::Completed { on: date() }).unwrap(),
            r#"{"status":"completed","on":"2026-08-07"}"#
        );
        assert_eq!(
            serde_json::to_string(&Completion::Open).unwrap(),
            r#"{"status":"open"}"#
        );
    }
}
