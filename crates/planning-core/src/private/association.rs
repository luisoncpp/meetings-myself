use super::clock::Clock;
use super::domain_error::DomainError;
use super::ids::{AssociationId, GoalId, HabitId, TaskId, ValueId};
use super::lifecycle::Lifecycle;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum AssociationEnd {
    Value(ValueId),
    Goal(GoalId),
    Habit(HabitId),
    Task(TaskId),
}

impl AssociationEnd {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Value(_) => "value",
            Self::Goal(_) => "goal",
            Self::Habit(_) => "habit",
            Self::Task(_) => "task",
        }
    }

    /// Canonical sort order. Storing every link in one direction turns duplicate
    /// detection into equality instead of a two-way search.
    fn rank(&self) -> u8 {
        match self {
            Self::Value(_) => 0,
            Self::Goal(_) => 1,
            Self::Habit(_) => 2,
            Self::Task(_) => 3,
        }
    }
}

pub struct Link<'a> {
    pub left: AssociationEnd,
    pub right: AssociationEnd,
    pub clock: &'a dyn Clock,
}

/// A many-to-many relevance link. Never implies ownership: archiving one side
/// leaves the other untouched and keeps the link dormant (ADR 0002).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Association {
    pub id: AssociationId,
    pub left: AssociationEnd,
    pub right: AssociationEnd,
    pub lifecycle: Lifecycle,
    pub created_at: DateTime<Utc>,
}

/// The only pairs CONTEXT.md defines, expressed once so the rule has one home.
const SUPPORTED: [(&str, &str); 4] = [
    ("value", "goal"),
    ("goal", "habit"),
    ("goal", "task"),
    ("habit", "task"),
];

impl Association {
    pub fn link(request: Link<'_>) -> Result<Self, DomainError> {
        let (left, right) = canonical(request.left, request.right);
        let pair = (left.kind(), right.kind());
        if !SUPPORTED.contains(&pair) {
            return Err(DomainError::UnsupportedAssociation {
                left: pair.0,
                right: pair.1,
            });
        }
        Ok(Self {
            id: AssociationId::generate(),
            left,
            right,
            lifecycle: Lifecycle::Active,
            created_at: request.clock.now(),
        })
    }

    pub fn touches(&self, end: &AssociationEnd) -> bool {
        self.left == *end || self.right == *end
    }

    pub fn other_side(&self, end: &AssociationEnd) -> Option<&AssociationEnd> {
        if self.left == *end {
            return Some(&self.right);
        }
        if self.right == *end {
            return Some(&self.left);
        }
        None
    }
}

fn canonical(left: AssociationEnd, right: AssociationEnd) -> (AssociationEnd, AssociationEnd) {
    if left.rank() <= right.rank() {
        return (left, right);
    }
    (right, left)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::clock::FixedClock;
    use chrono::TimeZone;

    fn clock() -> FixedClock {
        FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap())
    }

    fn goal() -> AssociationEnd {
        AssociationEnd::Goal(GoalId::new("g1"))
    }

    fn value() -> AssociationEnd {
        AssociationEnd::Value(ValueId::new("v1"))
    }

    #[test]
    fn supported_pairs_link_in_either_direction_and_normalize() {
        let forward = Association::link(Link {
            left: value(),
            right: goal(),
            clock: &clock(),
        })
        .unwrap();
        let backward = Association::link(Link {
            left: goal(),
            right: value(),
            clock: &clock(),
        })
        .unwrap();

        assert_eq!(forward.left, value(), "Value sorts before Goal");
        assert_eq!(forward.right, goal());
        assert_eq!(
            (backward.left.clone(), backward.right.clone()),
            (forward.left.clone(), forward.right.clone()),
            "direction must not create a second distinct link"
        );
    }

    #[test]
    fn unsupported_pairs_are_rejected() {
        let error = Association::link(Link {
            left: value(),
            right: AssociationEnd::Task(TaskId::new("t1")),
            clock: &clock(),
        })
        .unwrap_err();
        assert_eq!(
            error,
            DomainError::UnsupportedAssociation {
                left: "value",
                right: "task"
            }
        );

        assert!(Association::link(Link {
            left: goal(),
            right: goal(),
            clock: &clock()
        })
        .is_err());
    }

    #[test]
    fn a_new_link_is_active() {
        let link = Association::link(Link {
            left: value(),
            right: goal(),
            clock: &clock(),
        })
        .unwrap();
        assert_eq!(link.lifecycle, Lifecycle::Active);
    }

    #[test]
    fn a_link_can_report_the_other_side() {
        let link = Association::link(Link {
            left: value(),
            right: goal(),
            clock: &clock(),
        })
        .unwrap();
        assert!(link.touches(&goal()));
        assert_eq!(link.other_side(&goal()), Some(&value()));
        assert_eq!(
            link.other_side(&AssociationEnd::Task(TaskId::new("t9"))),
            None
        );
    }
}
