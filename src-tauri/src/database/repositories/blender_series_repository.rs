use crate::database::BlenderSeries;

use sqlx::SqlitePool;

pub struct BlenderSeriesRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> BlenderSeriesRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, blender_series: &BlenderSeries) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blender_series (id, is_collapsed, series, config_directory_path, created, modified) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(config_directory_path) DO NOTHING",
            blender_series.id,
            blender_series.is_collapsed,
            blender_series.series,
            blender_series.config_directory_path,
            blender_series.created,
            blender_series.modified
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch(
        &self,
        id: Option<String>,
        limit: Option<i64>,
        series: Option<String>,
        is_mapped: Option<bool>,
    ) -> Result<Vec<BlenderSeries>, sqlx::Error> {
        if let Some(id) = id {
            let item =
                sqlx::query_as::<_, BlenderSeries>("SELECT * FROM blender_series WHERE id = ?")
                    .bind(id)
                    .fetch_all(self.pool)
                    .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, BlenderSeries>("SELECT * FROM blender_series LIMIT ?")
                .bind(limit)
                .fetch_all(self.pool)
                .await
        } else if let Some(series) = series {
            let item =
                sqlx::query_as::<_, BlenderSeries>("SELECT * FROM blender_series WHERE series = ?")
                    .bind(series)
                    .fetch_all(self.pool)
                    .await?;
            Ok(item)
        } else if is_mapped.is_some_and(|x| x == true) {
            let item = sqlx::query_as::<_, BlenderSeries>(
                "SELECT blender_series.* FROM blender_series JOIN blend_file_blender_series ON blender_series.id = blend_file_blender_series.blender_series_id WHERE blend_file_blender_series.blender_series_id IS NOT NULL",
            )
            .bind(series)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, BlenderSeries>("SELECT * FROM blender_series")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(&self, blender_series: &BlenderSeries) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blender_series SET is_collapsed = ?, series = ?, config_directory_path = ?, modified = CURRENT_TIMESTAMP WHERE id = ?",
            blender_series.is_collapsed,
            blender_series.series,
            blender_series.config_directory_path,
            blender_series.id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blender_series WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
