use super::database::Database;
use super::error::StoreError;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use planning_core::Clock;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use surrealdb::types::SurrealValue;

/// Settings that must be identical on every device. Stored in the synchronized
/// database, not the device settings file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeSettings {
    pub home_zone: Option<Tz>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, SurrealValue)]
struct StoredHomeSettings {
    home_zone: Option<String>,
    updated_at: DateTime<Utc>,
}

fn parse_zone(name: &str) -> Result<Tz, StoreError> {
    Tz::from_str(name).map_err(|_| StoreError::Database(format!("unknown time zone: {name}")))
}

fn to_stored(settings: &HomeSettings) -> StoredHomeSettings {
    StoredHomeSettings {
        home_zone: settings.home_zone.map(|zone| zone.name().to_string()),
        updated_at: settings.updated_at,
    }
}

fn from_stored(stored: StoredHomeSettings) -> Result<HomeSettings, StoreError> {
    let home_zone = match stored.home_zone {
        Some(name) => Some(parse_zone(&name)?),
        None => None,
    };
    Ok(HomeSettings {
        home_zone,
        updated_at: stored.updated_at,
    })
}

fn empty_home_settings() -> HomeSettings {
    HomeSettings {
        home_zone: None,
        updated_at: DateTime::<Utc>::MIN_UTC,
    }
}

fn settings_not_found(error: &surrealdb::Error) -> bool {
    error.to_string().contains("does not exist")
}

pub struct SetZone<'a> {
    pub zone: Tz,
    pub clock: &'a dyn Clock,
}

pub struct HomeSettingsRepository;

impl HomeSettingsRepository {
    const TABLE: &'static str = "settings";
    const RECORD: &'static str = "home";

    pub async fn load(database: &Database) -> Result<HomeSettings, StoreError> {
        let result: Result<Option<StoredHomeSettings>, surrealdb::Error> =
            database.inner().select((Self::TABLE, Self::RECORD)).await;
        match result {
            Ok(Some(stored)) => from_stored(stored),
            Ok(None) => Ok(empty_home_settings()),
            Err(error) if settings_not_found(&error) => Ok(empty_home_settings()),
            Err(error) => Err(error.into()),
        }
    }

    /// Upserts the single settings record. There is exactly one, by construction,
    /// so a second device writing a zone replaces rather than duplicates it.
    pub async fn set_zone(
        database: &Database,
        request: SetZone<'_>,
    ) -> Result<HomeSettings, StoreError> {
        let settings = HomeSettings {
            home_zone: Some(request.zone),
            updated_at: request.clock.now(),
        };
        let stored = to_stored(&settings);
        let saved: Option<StoredHomeSettings> = database
            .inner()
            .upsert((Self::TABLE, Self::RECORD))
            .content(stored)
            .await?;
        match saved {
            Some(stored) => from_stored(stored),
            None => Ok(settings),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use chrono_tz::Tz;
    use planning_core::{Clock, FixedClock};
    use tempfile::TempDir;

    use super::super::database::Database;

    async fn database() -> (TempDir, Database) {
        let folder = TempDir::new().unwrap();
        let database = Database::open(folder.path()).await.unwrap();
        (folder, database)
    }

    #[tokio::test]
    async fn the_home_zone_starts_unset() {
        let (_folder, database) = database().await;
        let settings = HomeSettingsRepository::load(&database).await.unwrap();
        assert_eq!(settings.home_zone, None);
    }

    #[tokio::test]
    async fn setting_the_zone_persists_it_with_its_timestamp() {
        let (_folder, database) = database().await;
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap());

        let saved = HomeSettingsRepository::set_zone(
            &database,
            SetZone {
                zone: Tz::Europe__Madrid,
                clock: &clock,
            },
        )
        .await
        .unwrap();
        assert_eq!(saved.home_zone, Some(Tz::Europe__Madrid));
        assert_eq!(saved.updated_at, clock.now());

        let reloaded = HomeSettingsRepository::load(&database).await.unwrap();
        assert_eq!(reloaded.home_zone, Some(Tz::Europe__Madrid));
    }

    #[tokio::test]
    async fn changing_the_zone_replaces_it_rather_than_appending() {
        let (_folder, database) = database().await;
        let clock = FixedClock::at(Utc.with_ymd_and_hms(2026, 8, 7, 9, 0, 0).unwrap());
        for zone in [Tz::Europe__Madrid, Tz::America__Los_Angeles] {
            HomeSettingsRepository::set_zone(&database, SetZone { zone, clock: &clock })
                .await
                .unwrap();
        }
        let reloaded = HomeSettingsRepository::load(&database).await.unwrap();
        assert_eq!(reloaded.home_zone, Some(Tz::America__Los_Angeles));
    }
}
