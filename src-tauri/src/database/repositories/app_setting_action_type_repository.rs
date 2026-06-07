use crate::database::AppSettingActionType;

use sqlx::SqlitePool;

pub struct AppSettingActionTypeRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AppSettingActionTypeRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, e: &AppSettingActionType) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO 
            app_setting_action_type 
            (
            id, 
            code, 
            name, 
            description, 
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
            ?
            )",
            e.id,
            e.code,
            e.name,
            e.description,
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
    ) -> Result<Vec<AppSettingActionType>, sqlx::Error> {
        if let Some(id) = id {
            let item = sqlx::query_as::<_, AppSettingActionType>(
                "SELECT * FROM app_setting_action_type WHERE id = ?",
            )
            .bind(id)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, AppSettingActionType>(
                "SELECT * FROM app_setting_action_type LIMIT ?",
            )
            .bind(limit)
            .fetch_all(self.pool)
            .await
        } else if let Some(code) = code {
            let item = sqlx::query_as::<_, AppSettingActionType>(
                "SELECT * FROM app_setting_action_type WHERE code = ?",
            )
            .bind(code)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, AppSettingActionType>("SELECT * FROM app_setting_action_type")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(&self, e: &AppSettingActionType) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE 
            app_setting_action_type 
            SET 
            code = ?, 
            name = ?, 
            description = ?, 
            modified = CURRENT_TIMESTAMP 
            WHERE id = ?",
            e.code,
            e.name,
            e.description,
            e.id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM app_setting_action_type WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
