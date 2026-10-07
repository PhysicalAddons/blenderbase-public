use sqlx::SqlitePool;

/// Counters Blenderbase keeps for the achievements, and the snapshots of what is installed that
/// some rules read.
pub struct ActivityCounterRepository<'a> {
    pub pool: &'a SqlitePool,
}

/// What a `blender_version` row says about its build, enough to tell LTS, Stable, Beta and
/// Alpha apart.
#[derive(Default, Clone, Debug, sqlx::FromRow)]
pub struct InstalledBuild {
    pub release_cycle: Option<String>,
    pub risk_id: Option<String>,
    pub series: Option<String>,
}

impl<'a> ActivityCounterRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Adds `delta` to a counter, creating it at zero first.
    pub async fn add(&self, kind: &str, delta: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO activity_counter (kind, value) VALUES (?, ?)
             ON CONFLICT(kind) DO UPDATE SET value = activity_counter.value + excluded.value, modified = CURRENT_TIMESTAMP",
        )
        .bind(kind)
        .bind(delta)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch_all(&self) -> Result<Vec<(String, i64)>, sqlx::Error> {
        sqlx::query_as::<_, (String, i64)>("SELECT kind, value FROM activity_counter ORDER BY kind")
            .fetch_all(self.pool)
            .await
    }

    /// How many different addons of one kind (`addon`, `extension`, `core`) are installed,
    /// counting an addon present in several series once.
    pub async fn count_distinct_addons(&self, kind: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT COALESCE(functional_name, name, id)) FROM addon WHERE variant_type = ?",
        )
        .bind(kind)
        .fetch_one(self.pool)
        .await
    }

    /// The builds whose download finished: what is installed right now.
    pub async fn fetch_installed_builds(&self) -> Result<Vec<InstalledBuild>, sqlx::Error> {
        sqlx::query_as::<_, InstalledBuild>(
            "SELECT release_cycle, risk_id, series FROM blender_version WHERE download_status_type_id = 4",
        )
        .fetch_all(self.pool)
        .await
    }
}
