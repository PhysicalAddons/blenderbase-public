use crate::database::{
    BlenderSession, BlenderSessionProgress, BlenderSessionStart, BlenderSessionTotals,
    BlenderVersionTime,
};

use sqlx::SqlitePool;

pub struct BlenderSessionRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> BlenderSessionRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn fetch_by_id(&self, id: &str) -> Result<Option<BlenderSession>, sqlx::Error> {
        sqlx::query_as::<_, BlenderSession>("SELECT * FROM blender_session WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    /// One read of a log, in one transaction: the row is inserted on first sight (`start`
    /// present) or brought up to date, and every counter delta is added. An import interrupted
    /// halfway therefore never counts a chunk twice: the offset moves with the counters.
    pub async fn apply_import(
        &self,
        id: &str,
        start: Option<&BlenderSessionStart>,
        progress: &BlenderSessionProgress,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        match start {
            Some(s) => {
                sqlx::query(
                    "INSERT INTO blender_session (id, blender_version_id, version, series, build_hash, binary_path, pid, started_at, ended_at, open_seconds, active_seconds, is_finished, is_clean_exit, log_file_name, import_offset)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(id) DO UPDATE SET
                        ended_at = excluded.ended_at,
                        open_seconds = excluded.open_seconds,
                        active_seconds = blender_session.active_seconds + excluded.active_seconds,
                        is_finished = excluded.is_finished,
                        is_clean_exit = excluded.is_clean_exit,
                        import_offset = excluded.import_offset,
                        modified = CURRENT_TIMESTAMP",
                )
                .bind(id)
                .bind(&s.blender_version_id)
                .bind(&s.version)
                .bind(&s.series)
                .bind(&s.build_hash)
                .bind(&s.binary_path)
                .bind(s.pid)
                .bind(&s.started_at)
                .bind(&progress.ended_at)
                .bind(progress.open_seconds)
                .bind(progress.active_seconds_delta)
                .bind(progress.is_finished)
                .bind(progress.is_clean_exit)
                .bind(&s.log_file_name)
                .bind(progress.import_offset)
                .execute(&mut *tx)
                .await?;
            }
            None => {
                sqlx::query(
                    "UPDATE blender_session SET
                        ended_at = ?,
                        open_seconds = ?,
                        active_seconds = active_seconds + ?,
                        is_finished = ?,
                        is_clean_exit = ?,
                        import_offset = ?,
                        modified = CURRENT_TIMESTAMP
                     WHERE id = ?",
                )
                .bind(&progress.ended_at)
                .bind(progress.open_seconds)
                .bind(progress.active_seconds_delta)
                .bind(progress.is_finished)
                .bind(progress.is_clean_exit)
                .bind(progress.import_offset)
                .bind(id)
                .execute(&mut *tx)
                .await?;
            }
        }
        for (kind, value) in &progress.counters {
            if *value == 0 {
                continue;
            }
            sqlx::query(
                "INSERT INTO blender_session_counter (blender_session_id, kind, value) VALUES (?, ?, ?)
                 ON CONFLICT(blender_session_id, kind) DO UPDATE SET value = blender_session_counter.value + excluded.value",
            )
            .bind(id)
            .bind(kind)
            .bind(value)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await
    }

    /// Hours per version, most used first, over the sessions started at or after `since`
    /// (UTC ISO 8601; empty for all time). Sessions whose version is gone are one group with a
    /// NULL id, so their time still counts toward the total.
    pub async fn fetch_version_time(&self, since: &str) -> Result<Vec<BlenderVersionTime>, sqlx::Error> {
        sqlx::query_as::<_, BlenderVersionTime>(
            "SELECT
                blender_version_id,
                COALESCE(SUM(open_seconds), 0)   AS open_seconds,
                COALESCE(SUM(active_seconds), 0) AS active_seconds,
                COUNT(*)                         AS sessions,
                COALESCE(MAX(started_at), '')    AS last_used
             FROM blender_session
             WHERE started_at >= ?
             GROUP BY blender_version_id
             ORDER BY open_seconds DESC",
        )
        .bind(since)
        .fetch_all(self.pool)
        .await
    }

    /// The whole table in a handful of numbers, for the figures row and the achievement rules.
    pub async fn fetch_totals(&self, since: &str) -> Result<BlenderSessionTotals, sqlx::Error> {
        sqlx::query_as::<_, BlenderSessionTotals>(
            "SELECT
                COALESCE(SUM(open_seconds), 0)   AS open_seconds,
                COALESCE(SUM(active_seconds), 0) AS active_seconds,
                COALESCE(MAX(open_seconds), 0)   AS longest_session_seconds,
                COUNT(*)                         AS sessions,
                COALESCE(SUM(CASE WHEN is_finished = 1 THEN 1 ELSE 0 END), 0) AS finished_sessions,
                COALESCE(SUM(CASE WHEN is_finished = 1 AND is_clean_exit = 0 THEN 1 ELSE 0 END), 0) AS unclean_exits,
                COALESCE(SUM(CASE WHEN version LIKE '% Alpha%' THEN 1 ELSE 0 END), 0) AS alpha_sessions,
                COALESCE(MAX(CASE WHEN is_finished = 1 THEN ended_at ELSE '' END), '') AS latest_end
             FROM blender_session
             WHERE started_at >= ?",
        )
        .bind(since)
        .fetch_one(self.pool)
        .await
    }

    /// The most time any one series has: Loyal.
    pub async fn fetch_series_open_max(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(t), 0) FROM (SELECT SUM(open_seconds) AS t FROM blender_session GROUP BY series)",
        )
        .fetch_one(self.pool)
        .await
    }

    /// The most time any one installed version has.
    pub async fn fetch_version_open_max(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(t), 0) FROM (SELECT SUM(open_seconds) AS t FROM blender_session WHERE blender_version_id IS NOT NULL GROUP BY blender_version_id)",
        )
        .fetch_one(self.pool)
        .await
    }

    /// Every counter kind summed over the sessions started at or after `since` (empty for all).
    pub async fn fetch_counter_totals(&self, since: &str) -> Result<Vec<(String, i64)>, sqlx::Error> {
        sqlx::query_as::<_, (String, i64)>(
            "SELECT c.kind, COALESCE(SUM(c.value), 0)
             FROM blender_session_counter c
             JOIN blender_session s ON s.id = c.blender_session_id
             WHERE s.started_at >= ?
             GROUP BY c.kind
             ORDER BY c.kind",
        )
        .bind(since)
        .fetch_all(self.pool)
        .await
    }

    /// How many different installed versions were started at or after `since`: Version Hopper.
    pub async fn fetch_distinct_versions_since(&self, since: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT blender_version_id) FROM blender_session WHERE blender_version_id IS NOT NULL AND started_at >= ?",
        )
        .bind(since)
        .fetch_one(self.pool)
        .await
    }

    /// Start and end of every session that has an end: the Night Shift rule reads them in local time.
    pub async fn fetch_spans(&self) -> Result<Vec<(String, String)>, sqlx::Error> {
        sqlx::query_as::<_, (String, String)>(
            "SELECT started_at, ended_at FROM blender_session WHERE ended_at <> ''",
        )
        .fetch_all(self.pool)
        .await
    }

    /// Links sessions that have no version yet (or lost it) to a version again: run after a
    /// Blender install so a reinstalled build gets its hours back.
    pub async fn set_blender_version(
        &self,
        id: &str,
        blender_version_id: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE blender_session SET blender_version_id = ?, modified = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(blender_version_id)
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch_unmatched(&self) -> Result<Vec<BlenderSession>, sqlx::Error> {
        sqlx::query_as::<_, BlenderSession>(
            "SELECT * FROM blender_session WHERE blender_version_id IS NULL",
        )
        .fetch_all(self.pool)
        .await
    }
}
