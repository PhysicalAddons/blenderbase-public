use crate::database::BlenderInstallationLocation;

use sqlx::SqlitePool;

pub struct BlenderInstallationLocationRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> BlenderInstallationLocationRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(
        &self,
        blender_installation_location: &BlenderInstallationLocation,
    ) -> Result<(), sqlx::Error> {
        // `is_confirmed` is stored too: a location registered from the
        // first-download prompt or by the sweep is confirmed from the start,
        // and leaving it at the column default made the prompt come back.
        sqlx::query(
            "INSERT INTO blender_installation_location
            (
            id,
            is_default,
            full_control,
            modify,
            read_and_execute,
            list_folder_contents,
            read,
            write,
            special_permissions,
            directory_path,
            is_confirmed,
            created_by,
            created,
            modified
            ) VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )
            ON CONFLICT(directory_path) DO NOTHING",
        )
        .bind(&blender_installation_location.id)
        .bind(blender_installation_location.is_default)
        .bind(blender_installation_location.full_control)
        .bind(blender_installation_location.modify)
        .bind(blender_installation_location.read_and_execute)
        .bind(blender_installation_location.list_folder_contents)
        .bind(blender_installation_location.read)
        .bind(blender_installation_location.write)
        .bind(blender_installation_location.special_permissions)
        .bind(&blender_installation_location.directory_path)
        .bind(blender_installation_location.is_confirmed)
        .bind(&blender_installation_location.created_by)
        .bind(&blender_installation_location.created)
        .bind(&blender_installation_location.modified)
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch(
        &self,
        id: Option<String>,
        limit: Option<i64>,
        directory_path: Option<String>,
        is_default: Option<bool>,
    ) -> Result<Vec<BlenderInstallationLocation>, sqlx::Error> {
        if let Some(id) = id {
            let item = sqlx::query_as::<_, BlenderInstallationLocation>(
                "SELECT * FROM blender_installation_location WHERE id = ?",
            )
            .bind(id)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, BlenderInstallationLocation>(
                "SELECT * FROM blender_installation_location LIMIT ?",
            )
            .bind(limit)
            .fetch_all(self.pool)
            .await
        } else if let Some(file_path) = directory_path {
            let item = sqlx::query_as::<_, BlenderInstallationLocation>(
                "SELECT * FROM blender_installation_location WHERE directory_path = ?",
            )
            .bind(file_path)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(is_default) = is_default {
            let item = sqlx::query_as::<_, BlenderInstallationLocation>(
                "SELECT * FROM blender_installation_location WHERE is_default = ?",
            )
            .bind(is_default)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, BlenderInstallationLocation>(
                "SELECT * FROM blender_installation_location",
            )
            .fetch_all(self.pool)
            .await
        }
    }

    pub async fn update(
        &self,
        blender_installation_location: &BlenderInstallationLocation,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blender_installation_location 
            SET 
            is_default = ?,
            full_control = ?,
            modify = ?,
            read_and_execute = ?,
            list_folder_contents = ?,
            read = ?,
            write = ?,
            special_permissions = ?,
            directory_path = ?,
            created_by = ?,
            modified = CURRENT_TIMESTAMP
            WHERE id = ?",
            blender_installation_location.is_default,
            blender_installation_location.full_control,
            blender_installation_location.modify,
            blender_installation_location.read_and_execute,
            blender_installation_location.list_folder_contents,
            blender_installation_location.read,
            blender_installation_location.write,
            blender_installation_location.special_permissions,
            blender_installation_location.directory_path,
            blender_installation_location.created_by,
            blender_installation_location.id,
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blender_installation_location WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    /// Marks a location as confirmed and default, moving it to `directory_path` when the user
    /// picked a different folder in the prompt. Other locations lose their default flag.
    pub async fn confirm(
        &self,
        id: &str,
        directory_path: &str,
        permissions: &crate::core::PermissionDetails,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE blender_installation_location SET is_default = 0 WHERE id != ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        sqlx::query(
            "UPDATE blender_installation_location SET
                directory_path = ?, is_confirmed = 1, is_default = 1,
                full_control = ?, modify = ?, read_and_execute = ?, list_folder_contents = ?,
                read = ?, write = ?, special_permissions = ?, modified = CURRENT_TIMESTAMP
            WHERE id = ?",
        )
        .bind(directory_path)
        .bind(permissions.full_control)
        .bind(permissions.modify)
        .bind(permissions.read_and_execute)
        .bind(permissions.list_folder_contents)
        .bind(permissions.read)
        .bind(permissions.write)
        .bind(permissions.special_permissions)
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
