use crate::database::Addon;

use sqlx::SqlitePool;

pub struct AddonRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AddonRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, addon: &Addon) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO addon
            (
            id,
            is_enabled,
            is_symbolic_link,
            main_python_file_path,
            installation_directory,
            variant_type,
            functional_name,
            name,
            author,
            version,
            blender_version,
            location,
            description,
            warning,
            documentation_url,
            tracker_url,
            support,
            category,
            parent_blender_version_id,
            created,
            modified
            ) VALUES ( 
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?
            ) ON CONFLICT(main_python_file_path) DO NOTHING",
            addon.id,
            addon.is_enabled,
            addon.is_symbolic_link,
            addon.main_python_file_path,
            addon.installation_directory,
            addon.variant_type,
            addon.functional_name,
            addon.name,
            addon.author,
            addon.version,
            addon.blender_version,
            addon.location,
            addon.description,
            addon.warning,
            addon.documentation_url,
            addon.tracker_url,
            addon.support,
            addon.category,
            addon.parent_blender_version_id,
            addon.created,
            addon.modified
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch(
        &self,
        id: Option<String>,
        limit: Option<i64>,
        variant_type: Option<String>,
    ) -> Result<Vec<Addon>, sqlx::Error> {
        if let Some(id) = id {
            let item = sqlx::query_as::<_, Addon>("SELECT * FROM addon WHERE id = ?")
                .bind(id)
                .fetch_all(self.pool)
                .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, Addon>("SELECT * FROM addon LIMIT ?")
                .bind(limit)
                .fetch_all(self.pool)
                .await
        } else if let Some(file_path) = variant_type {
            let item = sqlx::query_as::<_, Addon>("SELECT * FROM addon WHERE variant_type = ?")
                .bind(file_path)
                .fetch_all(self.pool)
                .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, Addon>("SELECT * FROM addon")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(&self, addon: &Addon) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE addon 
            SET 
            is_enabled = ?,
            is_symbolic_link = ?,
            main_python_file_path = ?,
            installation_directory = ?,
            variant_type = ?,
            functional_name = ?,
            name = ?,
            author = ?,
            version = ?,
            blender_version = ?,
            location = ?,
            description = ?,
            warning = ?,
            documentation_url = ?,
            tracker_url = ?,
            support = ?,
            category = ?,
            parent_blender_version_id = ?,
            modified = CURRENT_TIMESTAMP
            WHERE id = ?",
            addon.is_enabled,
            addon.is_symbolic_link,
            addon.main_python_file_path,
            addon.installation_directory,
            addon.variant_type,
            addon.functional_name,
            addon.name,
            addon.author,
            addon.version,
            addon.blender_version,
            addon.location,
            addon.description,
            addon.warning,
            addon.documentation_url,
            addon.tracker_url,
            addon.support,
            addon.category,
            addon.parent_blender_version_id,
            addon.id,
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM addon WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
