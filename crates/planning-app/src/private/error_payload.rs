use super::error::AppError;
use planning_core::DomainError;
use planning_reports::ReportError;
use planning_store::StoreError;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorPayload {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, String>>,
}

impl AppErrorPayload {
    pub fn to_ipc_string(self) -> String {
        serde_json::to_string(&self).expect("error payload serializes")
    }
}

pub fn app_error_payload(error: AppError) -> AppErrorPayload {
    match error {
        AppError::Store(store) => store_payload(store),
        AppError::Domain(domain) => domain_payload(domain),
        AppError::Report(report) => report_payload(report),
        AppError::NotReady(_) => AppErrorPayload {
            code: "notReady".into(),
            params: None,
        },
        AppError::NoDatabase => AppErrorPayload {
            code: "noDatabase".into(),
            params: None,
        },
        AppError::NotFound { table, id } => AppErrorPayload {
            code: "notFound".into(),
            params: Some(HashMap::from([
                ("table".into(), table.into()),
                ("id".into(), id),
            ])),
        },
        AppError::NotSelectable { reason } => AppErrorPayload {
            code: "notSelectable".into(),
            params: Some(HashMap::from([("reason".into(), reason.into())])),
        },
        AppError::InvalidOrder => AppErrorPayload {
            code: "invalidOrder".into(),
            params: None,
        },
        AppError::InvalidZone(zone) => AppErrorPayload {
            code: "invalidZone".into(),
            params: Some(HashMap::from([("zone".into(), zone)])),
        },
        AppError::FutureCompletion => AppErrorPayload {
            code: "futureCompletion".into(),
            params: None,
        },
    }
}

fn store_payload(error: StoreError) -> AppErrorPayload {
    match error {
        StoreError::Io(io) => AppErrorPayload {
            code: "storeIo".into(),
            params: Some(HashMap::from([("path".into(), io.to_string())])),
        },
        StoreError::Corrupt { path, detail } => AppErrorPayload {
            code: "storeCorrupt".into(),
            params: Some(HashMap::from([
                ("path".into(), path.display().to_string()),
                ("detail".into(), detail),
            ])),
        },
        StoreError::NoConfigDirectory => AppErrorPayload {
            code: "noConfigDirectory".into(),
            params: None,
        },
        StoreError::Database(detail) => AppErrorPayload {
            code: "storeDatabase".into(),
            params: Some(HashMap::from([("detail".into(), detail)])),
        },
        StoreError::NotReady(_) => AppErrorPayload {
            code: "storeNotReady".into(),
            params: None,
        },
    }
}

fn domain_payload(error: DomainError) -> AppErrorPayload {
    match error {
        DomainError::BlankTitle => AppErrorPayload {
            code: "blankTitle".into(),
            params: None,
        },
        DomainError::UnsupportedAssociation { left, right } => AppErrorPayload {
            code: "unsupportedAssociation".into(),
            params: Some(HashMap::from([
                ("left".into(), left.into()),
                ("right".into(), right.into()),
            ])),
        },
        DomainError::EmptyCadence => AppErrorPayload {
            code: "emptyCadence".into(),
            params: None,
        },
        DomainError::InvalidMonthDay => AppErrorPayload {
            code: "invalidMonthDay".into(),
            params: None,
        },
    }
}

fn report_payload(error: ReportError) -> AppErrorPayload {
    match error {
        ReportError::Io(_) => AppErrorPayload {
            code: "reportIo".into(),
            params: None,
        },
        ReportError::MissingFrontMatter => AppErrorPayload {
            code: "reportMissingFrontMatter".into(),
            params: None,
        },
        ReportError::MalformedFrontMatter { detail } => AppErrorPayload {
            code: "reportMalformedFrontMatter".into(),
            params: Some(HashMap::from([("detail".into(), detail)])),
        },
        ReportError::UnsupportedSchema { found } => AppErrorPayload {
            code: "reportUnsupportedSchema".into(),
            params: Some(HashMap::from([("found".into(), found.to_string())])),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn not_found_serializes_a_stable_code() {
        let payload = app_error_payload(AppError::NotFound {
            table: "task",
            id: "abc".into(),
        });
        let value: Value = serde_json::from_str(&payload.to_ipc_string()).unwrap();
        assert_eq!(value["code"], "notFound");
        assert_eq!(value["params"]["table"], "task");
    }
}
