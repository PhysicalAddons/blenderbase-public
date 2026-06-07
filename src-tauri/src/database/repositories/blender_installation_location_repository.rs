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
        sqlx::query!(
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
            created_by,
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
            ?
            ) ON CONFLICT(directory_path) DO NOTHING",
            blender_installation_location.id,
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
            blender_installation_location.created,
            blender_installation_location.modified
        )
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
}
