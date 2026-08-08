use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("a title cannot be blank")]
    BlankTitle,

    #[error("{left} cannot be associated with {right}")]
    UnsupportedAssociation {
        left: &'static str,
        right: &'static str,
    },

    #[error("a habit cadence must include at least one weekday")]
    EmptyCadence,

    #[error("a monthly recurrence day must be between 1 and 31")]
    InvalidMonthDay,
}
