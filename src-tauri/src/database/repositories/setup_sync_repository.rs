use crate::database::SetupSync;

use sqlx::SqlitePool;

pub struct SetupSyncRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> SetupSyncRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn fetch(&self) -> Result<SetupSync, sqlx::Error> {
        sqlx::query_as::<_, SetupSync>(
            "SELECT id, folder_path, last_synced_hash, last_synced_at, created, modified FROM setup_sync WHERE id = 1",
        )
        .fetch_one(self.pool)
        .await
    }

    /// Changing the folder forgets what was synced: the new folder's file is news either way.
    pub async fn set_folder(&self, folder_path: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE setup_sync SET folder_path = ?, last_synced_hash = '', last_synced_at = '', modified = CURRENT_TIMESTAMP WHERE id = 1",
        )
        .bind(folder_path)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_synced(&self, content_hash: &str, at: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE setup_sync SET last_synced_hash = ?, last_synced_at = ?, modified = CURRENT_TIMESTAMP WHERE id = 1",
        )
        .bind(content_hash)
        .bind(at)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
