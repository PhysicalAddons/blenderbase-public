use crate::database::DownloadStatusType;

use sqlx::{QueryBuilder, SqlitePool};

pub struct DownloadStatusTypeRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> DownloadStatusTypeRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        download_status_type: &DownloadStatusType,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO download_status_type (code, name, description, parent_download_status_type_id, created, modified) VALUES (?, ?, ?, ?, ?, ?)",
            download_status_type.code,
            download_status_type.name,
            download_status_type.description,
            download_status_type.parent_download_status_type_id,
            download_status_type.created,
            download_status_type.modified
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch(
        &self,
        id: Option<i64>,
        limit: Option<i64>,
        codes: Option<Vec<String>>,
    ) -> Result<Vec<DownloadStatusType>, sqlx::Error> {
        if let Some(id) = id {
            let item = sqlx::query_as::<_, DownloadStatusType>(
                "SELECT * FROM download_status_type WHERE id = ?",
            )
            .bind(id)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, DownloadStatusType>("SELECT * FROM download_status_type LIMIT ?")
                .bind(limit)
                .fetch_all(self.pool)
                .await
        } else if let Some(codes) = codes {
            let mut qb = QueryBuilder::new("SELECT * FROM download_status_type WHERE code IN (");
            let mut separated = qb.separated(", ");
            for code in &codes {
                separated.push_bind(code.to_uppercase());
            }
            separated.push_unseparated(")");
            let items = qb
                .build_query_as::<DownloadStatusType>()
                .fetch_all(self.pool)
                .await?;
            Ok(items)
        } else {
            sqlx::query_as::<_, DownloadStatusType>("SELECT * FROM download_status_type")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(
        &self,
        download_status_type: &DownloadStatusType,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE download_status_type SET code = ?, name = ?, description = ?, parent_download_status_type_id = ?, modified = CURRENT_TIMESTAMP WHERE id = ?",
            download_status_type.code,
            download_status_type.name,
            download_status_type.description,
            download_status_type.parent_download_status_type_id,
            download_status_type.id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM download_status_type WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
