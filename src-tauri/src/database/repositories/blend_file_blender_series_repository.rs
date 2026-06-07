use crate::database::BlendFileBlenderSeries;

use sqlx::SqlitePool;

pub struct BlendFileBlenderSeriesRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> BlendFileBlenderSeriesRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        blend_file_blender_series: &BlendFileBlenderSeries,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blend_file_blender_series 
            (blend_file_id, blender_series_id, created, modified) VALUES (?, ?, ?, ?) ON CONFLICT(blend_file_id, blender_series_id) DO NOTHING",
            blend_file_blender_series.blend_file_id,
            blend_file_blender_series.blender_series_id,
            blend_file_blender_series.created,
            blend_file_blender_series.modified
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }
    pub async fn fetch(
        &self,
        blend_file_id: Option<String>,
        blender_series_id: Option<String>,
        _limit: Option<i64>,
    ) -> Result<Vec<BlendFileBlenderSeries>, sqlx::Error> {
        if let Some(blend_file_id) = blend_file_id {
            let item = sqlx::query_as::<_, BlendFileBlenderSeries>(
                "SELECT * FROM blend_file_blender_series WHERE blend_file_id = ?",
            )
            .bind(blend_file_id)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(blender_series_id) = blender_series_id {
            let item = sqlx::query_as::<_, BlendFileBlenderSeries>(
                "SELECT * FROM blend_file_blender_series WHERE blender_series_id = ?",
            )
            .bind(blender_series_id)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, BlendFileBlenderSeries>("SELECT * FROM blend_file_blender_series")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(
        &self,
        blend_file_blender_series: &BlendFileBlenderSeries,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blend_file_blender_series SET modified = CURRENT_TIMESTAMP WHERE blend_file_id = ? AND blender_series_id = ?",
            blend_file_blender_series.blend_file_id,
            blend_file_blender_series.blender_series_id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blend_file_blender_series WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
