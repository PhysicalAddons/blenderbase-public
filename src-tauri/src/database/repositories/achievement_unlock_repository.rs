use crate::database::AchievementUnlock;

use sqlx::SqlitePool;

pub struct AchievementUnlockRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AchievementUnlockRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn fetch_all(&self) -> Result<Vec<AchievementUnlock>, sqlx::Error> {
        sqlx::query_as::<_, AchievementUnlock>(
            "SELECT * FROM achievement_unlock ORDER BY unlocked_at DESC, created DESC",
        )
        .fetch_all(self.pool)
        .await
    }

    /// Records an unlock; a second record of the same achievement is ignored.
    pub async fn insert(
        &self,
        achievement_id: &str,
        unlocked_at: &str,
        blender_session_id: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO achievement_unlock (achievement_id, unlocked_at, blender_session_id, is_seen) VALUES (?, ?, ?, 0)
             ON CONFLICT(achievement_id) DO NOTHING",
        )
        .bind(achievement_id)
        .bind(unlocked_at)
        .bind(blender_session_id)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    /// Unlocks nobody has looked at yet: the title-bar badge.
    pub async fn count_unseen(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM achievement_unlock WHERE is_seen = 0")
            .fetch_one(self.pool)
            .await
    }

    pub async fn mark_all_seen(&self) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE achievement_unlock SET is_seen = 1 WHERE is_seen = 0")
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
