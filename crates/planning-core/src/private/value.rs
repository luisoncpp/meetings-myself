use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::ValueId;
use super::lifecycle::Lifecycle;
use super::task::clean_title;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct CreateValue<'a> {
    pub title: String,
    pub clock: &'a dyn Clock,
}

/// An enduring personal principle. Values are active or archived — never completed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Value {
    pub id: ValueId,
    pub title: String,
    pub lifecycle: Lifecycle,
    pub created_at: DateTime<Utc>,
}

impl Value {
    pub fn create(request: CreateValue<'_>) -> Result<Self, DomainError> {
        Ok(Self {
            id: ValueId::generate(),
            title: clean_title(request.title)?,
            lifecycle: Lifecycle::Active,
            created_at: request.clock.now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    #[test]
    fn a_title_alone_is_enough_to_create_a_value() {
        let value = Value::create(CreateValue {
            title: "Integrity".into(),
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(value.title, "Integrity");
        assert_eq!(value.lifecycle, Lifecycle::Active);
    }

    #[test]
    fn blank_titles_are_rejected_and_whitespace_is_trimmed() {
        assert_eq!(
            Value::create(CreateValue {
                title: "   ".into(),
                clock: &clock()
            }),
            Err(DomainError::BlankTitle)
        );
        let value = Value::create(CreateValue {
            title: "  Health  ".into(),
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(value.title, "Health");
    }
}
