use crate::database::AppSettingType;

use sqlx::SqlitePool;

pub struct AppSettingTypeRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AppSettingTypeRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, e: &AppSettingType) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO 
            app_setting_type 
            (
            id, 
            code, 
            name, 
            description, 
            parent_app_setting_type_id, 
            created, 
            modified
            ) 
            VALUES 
            (
            ?, 
            ?, 
            ?, 
            ?, 
            ?, 
            ?, 
            ?
            )",
            e.id,
            e.code,
            e.name,
            e.description,
            e.parent_app_setting_type_id,
            e.created,
            e.modified
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch(
        &self,
        id: Option<i64>,
        limit: Option<i64>,
        code: Option<String>,
    ) -> Result<Vec<AppSettingType>, sqlx::Error> {
        if let Some(id) = id {
            let item =
                sqlx::query_as::<_, AppSettingType>("SELECT * FROM app_setting_type WHERE id = ?")
                    .bind(id)
                    .fetch_all(self.pool)
                    .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, AppSettingType>("SELECT * FROM app_setting_type LIMIT ?")
                .bind(limit)
                .fetch_all(self.pool)
                .await
        } else if let Some(code) = code {
            let item = sqlx::query_as::<_, AppSettingType>(
                "SELECT * FROM app_setting_type WHERE code = ?",
            )
            .bind(code)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, AppSettingType>("SELECT * FROM app_setting_type")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(&self, e: &AppSettingType) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE 
            app_setting_type 
            SET 
            code = ?, 
            name = ?, 
            description = ?, 
            parent_app_setting_type_id = ?, 
            modified = CURRENT_TIMESTAMP 
            WHERE id = ?",
            e.code,
            e.name,
            e.description,
            e.parent_app_setting_type_id,
            e.id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM app_setting_type WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
