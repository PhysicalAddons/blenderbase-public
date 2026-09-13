use crate::database::Addon;

use sqlx::SqlitePool;

/// `is_symbolic_link` is declared TEXT, so SQLite stores the bound bool as '0'/'1'; the
/// cast lets sqlx decode it back into a bool.
macro_rules! addon_columns {
    () => {
        "id, CAST(is_enabled AS INTEGER) AS is_enabled, CAST(is_symbolic_link AS INTEGER) AS is_symbolic_link, main_python_file_path, installation_directory, variant_type, functional_name, name, author, version, blender_version, location, description, warning, documentation_url, tracker_url, support, category, parent_blender_version_id, created, modified"
    };
}

pub struct AddonRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AddonRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, addon: &Addon) -> Result<(), sqlx::Error> {
        self.insert_with(self.pool, addon).await
    }

    /// Same upsert, but on the given executor (a pool or an open transaction).
    pub async fn insert_with<'e, E>(&self, executor: E, addon: &Addon) -> Result<(), sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
    {
        sqlx::query(
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
            parent_blender_version_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(main_python_file_path, parent_blender_version_id) DO UPDATE SET
            is_enabled = excluded.is_enabled,
            is_symbolic_link = excluded.is_symbolic_link,
            installation_directory = excluded.installation_directory,
            variant_type = excluded.variant_type,
            functional_name = excluded.functional_name,
            name = excluded.name,
            author = excluded.author,
            version = excluded.version,
            blender_version = excluded.blender_version,
            location = excluded.location,
            description = excluded.description,
            warning = excluded.warning,
            documentation_url = excluded.documentation_url,
            tracker_url = excluded.tracker_url,
            support = excluded.support,
            category = excluded.category,
            modified = CURRENT_TIMESTAMP",
        )
        .bind(&addon.id)
        .bind(addon.is_enabled)
        .bind(addon.is_symbolic_link)
        .bind(&addon.main_python_file_path)
        .bind(&addon.installation_directory)
        .bind(&addon.variant_type)
        .bind(&addon.functional_name)
        .bind(&addon.name)
        .bind(&addon.author)
        .bind(&addon.version)
        .bind(&addon.blender_version)
        .bind(&addon.location)
        .bind(&addon.description)
        .bind(&addon.warning)
        .bind(&addon.documentation_url)
        .bind(&addon.tracker_url)
        .bind(&addon.support)
        .bind(&addon.category)
        .bind(&addon.parent_blender_version_id)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub async fn fetch(&self, id: Option<String>) -> Result<Vec<Addon>, sqlx::Error> {
        if let Some(id) = id {
            sqlx::query_as::<_, Addon>(concat!("SELECT ", addon_columns!(), " FROM addon WHERE id = ?"))
                .bind(id)
                .fetch_all(self.pool)
                .await
        } else {
            sqlx::query_as::<_, Addon>(concat!("SELECT ", addon_columns!(), " FROM addon"))
                .fetch_all(self.pool)
                .await
        }
    }

    /// Addons cached for one installed Blender version, ordered by kind and then by name.
    pub async fn fetch_by_blender_version(
        &self,
        blender_version_id: &str,
    ) -> Result<Vec<Addon>, sqlx::Error> {
        sqlx::query_as::<_, Addon>(concat!(
            "SELECT ",
            addon_columns!(),
            " FROM addon WHERE parent_blender_version_id = ?
            ORDER BY
                CASE variant_type WHEN 'extension' THEN 0 WHEN 'addon' THEN 1 ELSE 2 END,
                name COLLATE NOCASE"
        ))
        .bind(blender_version_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn update_is_enabled(&self, id: &str, is_enabled: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE addon SET is_enabled = ?, modified = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(is_enabled)
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        self.delete_with(self.pool, id).await
    }

    pub async fn delete_with<'e, E>(&self, executor: E, id: &str) -> Result<(), sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
    {
        sqlx::query("DELETE FROM addon WHERE id = ?")
            .bind(id)
            .execute(executor)
            .await?;
        Ok(())
    }
}
