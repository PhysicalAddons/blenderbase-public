use crate::database::BlenderVersionBuildType;

use sqlx::SqlitePool;

pub struct BlenderVersionBuildTypeRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> BlenderVersionBuildTypeRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        app_setting_type: &BlenderVersionBuildType,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blender_version_build_type (is_default, code, name, description, created, modified) VALUES (?, ?, ?, ?, ?, ?)",
            app_setting_type.code,
            app_setting_type.is_default,
            app_setting_type.name,
            app_setting_type.description,
            app_setting_type.created,
            app_setting_type.modified
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
    ) -> Result<Vec<BlenderVersionBuildType>, sqlx::Error> {
        if let Some(id) = id {
            let item = sqlx::query_as::<_, BlenderVersionBuildType>(
                "SELECT * FROM blender_version_build_type WHERE id = ?",
            )
            .bind(id)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, BlenderVersionBuildType>(
                "SELECT * FROM blender_version_build_type LIMIT ?",
            )
            .bind(limit)
            .fetch_all(self.pool)
            .await
        } else if let Some(code) = code {
            let item = sqlx::query_as::<_, BlenderVersionBuildType>(
                "SELECT * FROM blender_version_build_type WHERE code = ?",
            )
            .bind(code)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, BlenderVersionBuildType>("SELECT * FROM blender_version_build_type")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(
        &self,
        app_setting_type: &BlenderVersionBuildType,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blender_version_build_type SET is_default = ?, code = ?, name = ?, description = ?, modified = CURRENT_TIMESTAMP WHERE id = ?",
            app_setting_type.is_default,
            app_setting_type.code,
            app_setting_type.name,
            app_setting_type.description,
            app_setting_type.id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blender_version_build_type WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
