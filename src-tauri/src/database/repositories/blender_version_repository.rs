use crate::database::BlenderVersion;

use sqlx::SqlitePool;

pub struct BlenderVersionRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> BlenderVersionRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, blender_version: &BlenderVersion) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blender_version
            (
            id,
            is_default,
            custom_name,
            url,
            app,
            version,
            series,
            risk_id,
            branch,
            patch,
            patch_url,
            hash,
            hash_url,
            platform,
            architecture,
            bitness,
            file_mtime,
            file_name,
            file_size,
            file_extension,
            release_cycle,
            checksum,
            installation_directory_path,
            executable_file_path,
            blender_installation_location_id,
            download_status_type_id,
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
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?,
            ?
            ) 
            ON CONFLICT DO NOTHING",
            // ON CONFLICT(executable_file_path) DO NOTHING
            blender_version.id,
            blender_version.is_default,
            blender_version.custom_name,
            blender_version.url,
            blender_version.app,
            blender_version.version,
            blender_version.series,
            blender_version.risk_id,
            blender_version.branch,
            blender_version.patch,
            blender_version.patch_url,
            blender_version.hash,
            blender_version.hash_url,
            blender_version.platform,
            blender_version.architecture,
            blender_version.bitness,
            blender_version.file_mtime,
            blender_version.file_name,
            blender_version.file_size,
            blender_version.file_extension,
            blender_version.release_cycle,
            blender_version.checksum,
            blender_version.installation_directory_path,
            blender_version.executable_file_path,
            blender_version.blender_installation_location_id,
            blender_version.download_status_type_id,
            blender_version.created,
            blender_version.modified
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn fetch(
        &self,
        id: Option<String>,
        limit: Option<i64>,
        is_default: Option<bool>,
        executable_file_path: Option<String>,
        series: Option<String>,
    ) -> Result<Vec<BlenderVersion>, sqlx::Error> {
        if let Some(id) = id {
            let item =
                sqlx::query_as::<_, BlenderVersion>("SELECT * FROM blender_version WHERE id = ?")
                    .bind(id)
                    .fetch_all(self.pool)
                    .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, BlenderVersion>("SELECT * FROM blender_version LIMIT ?")
                .bind(limit)
                .fetch_all(self.pool)
                .await
        } else if let Some(is_default) = is_default {
            let item = sqlx::query_as::<_, BlenderVersion>(
                "SELECT * FROM blender_version WHERE is_default = ?",
            )
            .bind(is_default)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(file_path) = executable_file_path {
            let item = sqlx::query_as::<_, BlenderVersion>(
                "SELECT * FROM blender_version WHERE executable_file_path = ?",
            )
            .bind(file_path)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(series) = series {
            let item = sqlx::query_as::<_, BlenderVersion>(
                "SELECT * FROM blender_version WHERE series = ?",
            )
            .bind(series)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, BlenderVersion>("SELECT * FROM blender_version")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(&self, blender_version: &BlenderVersion) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blender_version 
            SET 
            is_default = ?,
            custom_name = ?,
            url = ?,
            app = ?,
            version = ?,
            series = ?,
            risk_id = ?,
            branch = ?,
            patch_url = ?,
            patch = ?,
            hash_url = ?,
            hash = ?,
            platform = ?,
            architecture = ?,
            bitness = ?,
            file_mtime = ?,
            file_name = ?,
            file_size = ?,
            file_extension = ?,
            release_cycle = ?,
            checksum = ?,
            installation_directory_path = ?,
            executable_file_path = ?,
            blender_installation_location_id = ?,
            download_status_type_id = ?,
            modified = CURRENT_TIMESTAMP
            WHERE id = ?",
            blender_version.is_default,
            blender_version.custom_name,
            blender_version.url,
            blender_version.app,
            blender_version.version,
            blender_version.series,
            blender_version.risk_id,
            blender_version.branch,
            blender_version.patch_url,
            blender_version.patch,
            blender_version.hash_url,
            blender_version.hash,
            blender_version.platform,
            blender_version.architecture,
            blender_version.bitness,
            blender_version.file_mtime,
            blender_version.file_name,
            blender_version.file_size,
            blender_version.file_extension,
            blender_version.release_cycle,
            blender_version.checksum,
            blender_version.installation_directory_path,
            blender_version.executable_file_path,
            blender_version.blender_installation_location_id,
            blender_version.download_status_type_id,
            blender_version.id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM blender_version WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
