use serde::{Deserialize, Serialize};

/// Manual assessment of Importance or Urgency. Unclassified is a real answer,
/// not a missing value — creating a Task from a title alone must be frictionless.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Classification {
    #[default]
    Unclassified,
    Low,
    High,
}
