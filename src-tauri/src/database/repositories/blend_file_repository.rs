use crate::database::BlendFile;

use sqlx::SqlitePool;

pub struct BlendFileRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> BlendFileRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, blend_file: &BlendFile) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blend_file 
            (id, file_path, file_name, file_size, created_datetime, modified_datetime, accessed_datetime, last_used_blender_version_id, created, modified) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(file_path) DO NOTHING",
            blend_file.id,
            blend_file.file_path,
            blend_file.file_name,
            blend_file.file_size,
            blend_file.created_datetime,
            blend_file.modified_datetime,
            blend_file.accessed_datetime,
            blend_file.last_used_blender_version_id,
            blend_file.created,
            blend_file.modified
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch(
        &self,
        id: Option<String>,
        limit: Option<i64>,
        file_path: Option<String>,
        blender_series_id: Option<String>,
    ) -> Result<Vec<BlendFile>, sqlx::Error> {
        if let Some(id) = id {
            let item = sqlx::query_as::<_, BlendFile>("SELECT * FROM blend_file WHERE id = ?")
                .bind(id)
                .fetch_all(self.pool)
                .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, BlendFile>("SELECT * FROM blend_file LIMIT ?")
                .bind(limit)
                .fetch_all(self.pool)
                .await
        } else if let Some(file_path) = file_path {
            let item =
                sqlx::query_as::<_, BlendFile>("SELECT * FROM blend_file WHERE file_path = ?")
                    .bind(file_path)
                    .fetch_all(self.pool)
                    .await?;
            Ok(item)
        } else if let Some(blender_series_id) = blender_series_id {
            let item =
                sqlx::query_as::<_, BlendFile>("SELECT blend_file.* FROM blend_file JOIN blend_file_blender_series ON blend_file.id = blend_file_blender_series.blend_file_id WHERE blend_file_blender_series.blend_file_id IS NOT NULL AND blend_file_blender_series.blender_series_id = ?")
                    .bind(blender_series_id)
                    .fetch_all(self.pool)
                    .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, BlendFile>("SELECT * FROM blend_file")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(&self, blend_file: &BlendFile) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blend_file SET file_path = ?, file_name = ?, file_size = ?, created_datetime = ?, modified_datetime = ?, accessed_datetime = ?, last_used_blender_version_id = ?, modified = CURRENT_TIMESTAMP WHERE id = ?",
            blend_file.file_path,
            blend_file.file_name,
            blend_file.file_size,
            blend_file.created_datetime,
            blend_file.modified_datetime,
            blend_file.accessed_datetime,
            blend_file.last_used_blender_version_id,
            blend_file.id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blend_file WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
