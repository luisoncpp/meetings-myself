use super::domain_error::DomainError;
use chrono::Weekday;
use serde::{Deserialize, Serialize};

const ORDER: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

const NAMES: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

/// A set of weekdays stored as a 7-bit mask but serialized as readable names,
/// because these values end up in Weekly Report front matter that humans edit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
pub struct WeekdaySet(u8);

impl WeekdaySet {
    pub fn from_weekdays(days: &[Weekday]) -> Self {
        let mut mask = 0u8;
        for day in days {
            mask |= 1 << position(*day);
        }
        Self(mask)
    }

    pub fn contains(&self, day: Weekday) -> bool {
        self.0 & (1 << position(day)) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Always in Monday-first order regardless of insertion order.
    pub fn weekdays(&self) -> Vec<Weekday> {
        ORDER
            .into_iter()
            .filter(|day| self.contains(*day))
            .collect()
    }
}

fn position(day: Weekday) -> usize {
    day.num_days_from_monday() as usize
}

impl From<WeekdaySet> for Vec<String> {
    fn from(set: WeekdaySet) -> Self {
        set.weekdays()
            .into_iter()
            .map(|day| NAMES[position(day)].to_string())
            .collect()
    }
}

impl TryFrom<Vec<String>> for WeekdaySet {
    type Error = DomainError;

    fn try_from(names: Vec<String>) -> Result<Self, Self::Error> {
        let mut days = Vec::new();
        for name in names {
            let index = NAMES
                .iter()
                .position(|candidate| *candidate == name)
                .ok_or(DomainError::EmptyCadence)?;
            days.push(ORDER[index]);
        }
        Ok(Self::from_weekdays(&days))
    }
}

/// The days on which a Habit is due. Changes apply from the next Daily Plan only
/// (ADR 0002).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Cadence {
    EveryDay,
    OnWeekdays { days: WeekdaySet },
}

impl Cadence {
    pub fn new_on_weekdays(days: &[Weekday]) -> Result<Self, DomainError> {
        let set = WeekdaySet::from_weekdays(days);
        if set.is_empty() {
            return Err(DomainError::EmptyCadence);
        }
        Ok(Self::OnWeekdays { days: set })
    }

    pub fn is_due(&self, day: Weekday) -> bool {
        match self {
            Self::EveryDay => true,
            Self::OnWeekdays { days } => days.contains(day),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_day_is_due_on_every_weekday() {
        for day in [Weekday::Mon, Weekday::Sat, Weekday::Sun] {
            assert!(Cadence::EveryDay.is_due(day));
        }
    }

    #[test]
    fn selected_weekdays_are_due_only_on_those_days() {
        let cadence = Cadence::new_on_weekdays(&[Weekday::Mon, Weekday::Wed]).unwrap();
        assert!(cadence.is_due(Weekday::Mon));
        assert!(cadence.is_due(Weekday::Wed));
        assert!(!cadence.is_due(Weekday::Tue));
        assert!(!cadence.is_due(Weekday::Sun));
    }

    #[test]
    fn an_empty_cadence_is_rejected_at_construction() {
        assert_eq!(
            Cadence::new_on_weekdays(&[]),
            Err(DomainError::EmptyCadence)
        );
    }

    #[test]
    fn duplicate_weekdays_collapse() {
        let cadence = Cadence::new_on_weekdays(&[Weekday::Mon, Weekday::Mon]).unwrap();
        let Cadence::OnWeekdays { days } = cadence else {
            panic!("expected OnWeekdays")
        };
        assert_eq!(days.weekdays(), vec![Weekday::Mon]);
    }

    #[test]
    fn weekday_sets_serialize_as_readable_day_names() {
        let days = WeekdaySet::from_weekdays(&[Weekday::Wed, Weekday::Mon]);
        assert_eq!(serde_json::to_string(&days).unwrap(), r#"["mon","wed"]"#);
        let parsed: WeekdaySet = serde_json::from_str(r#"["mon","wed"]"#).unwrap();
        assert_eq!(parsed, days);
    }
}
