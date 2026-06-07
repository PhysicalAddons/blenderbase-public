use crate::database::AppSetting;

use sqlx::SqlitePool;

pub struct AppSettingRepository<'a> {
    pub pool: &'a SqlitePool,
}

impl<'a> AppSettingRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, e: &AppSetting) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT 
            INTO app_setting 
            (
            code,
            name,
            description,
            disclaimer,
            is_enabled,
            is_read_on_app_launch,
            is_read_on_app_exit,
            default_int_value,
            int_value,
            min_int_value,
            max_int_value,
            is_signed_int_value,
            default_text_value,
            text_value,
            min_length_text_value,
            max_length_text_value,
            regex_format_text_value,
            default_range_int_value_from,
            default_range_int_value_to,
            range_int_value_from,
            range_int_value_to,
            min_range_int_value_from,
            min_range_int_value_to,
            max_range_int_value_from,
            max_range_int_value_to,
            is_signed_range_int_value_from,
            is_signed_range_int_value_to,
            default_range_text_value_from,
            default_range_text_value_to,
            range_text_value_from,
            range_text_value_to,
            min_length_range_text_value_from,
            max_length_range_text_value_from,
            min_length_range_text_value_to,
            max_length_range_text_value_to,
            regex_format_range_text_value_from,
            regex_format_range_text_value_to,
            control_title,
            parent_app_setting_id,
            app_setting_type_id,
            app_setting_action_type_id,
            measurement_unit_type_id,
            input_value_type_id,
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
            )",
            e.code,
            e.name,
            e.description,
            e.disclaimer,
            e.is_enabled,
            e.is_read_on_app_launch,
            e.is_read_on_app_exit,
            e.default_int_value,
            e.int_value,
            e.min_int_value,
            e.max_int_value,
            e.is_signed_int_value,
            e.default_text_value,
            e.text_value,
            e.min_length_text_value,
            e.max_length_text_value,
            e.regex_format_text_value,
            e.default_range_int_value_from,
            e.default_range_int_value_to,
            e.range_int_value_from,
            e.range_int_value_to,
            e.min_range_int_value_from,
            e.min_range_int_value_to,
            e.max_range_int_value_from,
            e.max_range_int_value_to,
            e.is_signed_range_int_value_from,
            e.is_signed_range_int_value_to,
            e.default_range_text_value_from,
            e.default_range_text_value_to,
            e.range_text_value_from,
            e.range_text_value_to,
            e.min_length_range_text_value_from,
            e.max_length_range_text_value_from,
            e.min_length_range_text_value_to,
            e.max_length_range_text_value_to,
            e.regex_format_range_text_value_from,
            e.regex_format_range_text_value_to,
            e.control_title,
            e.parent_app_setting_id,
            e.app_setting_type_id,
            e.app_setting_action_type_id,
            e.measurement_unit_type_id,
            e.input_value_type_id,
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
        is_read_on_app_launch: Option<bool>,
        app_setting_type_id: Option<i64>,
    ) -> Result<Vec<AppSetting>, sqlx::Error> {
        if let Some(id) = id {
            let item = sqlx::query_as::<_, AppSetting>("SELECT * FROM app_setting WHERE id = ?")
                .bind(id)
                .fetch_all(self.pool)
                .await?;
            Ok(item)
        } else if let Some(limit) = limit {
            sqlx::query_as::<_, AppSetting>("SELECT * FROM app_setting LIMIT ?")
                .bind(limit)
                .fetch_all(self.pool)
                .await
        } else if let Some(code) = code {
            let item = sqlx::query_as::<_, AppSetting>("SELECT * FROM app_setting WHERE code = ?")
                .bind(code)
                .fetch_all(self.pool)
                .await?;
            Ok(item)
        } else if let Some(is_read_on_app_launch) = is_read_on_app_launch {
            let item = sqlx::query_as::<_, AppSetting>(
                "SELECT * FROM app_setting WHERE is_read_on_app_launch = ?",
            )
            .bind(is_read_on_app_launch)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else if let Some(app_setting_type_id) = app_setting_type_id {
            let item = sqlx::query_as::<_, AppSetting>(
                "SELECT * FROM app_setting WHERE app_setting_type_id = ?",
            )
            .bind(app_setting_type_id)
            .fetch_all(self.pool)
            .await?;
            Ok(item)
        } else {
            sqlx::query_as::<_, AppSetting>("SELECT * FROM app_setting")
                .fetch_all(self.pool)
                .await
        }
    }

    pub async fn update(&self, e: &AppSetting) -> Result<(), sqlx::Error> {
        println!("SETTING UPDATE {:?}", e);
        sqlx::query!(
            "UPDATE 
            app_setting 
            SET 
            code = ?,
            name = ?,
            description = ?,
            disclaimer = ?,
            is_enabled = ?,
            is_read_on_app_launch = ?,
            is_read_on_app_exit = ?,
            default_int_value = ?,
            int_value = ?,
            min_int_value = ?,
            max_int_value = ?,
            is_signed_int_value = ?,
            default_text_value = ?,
            text_value = ?,
            min_length_text_value = ?,
            max_length_text_value = ?,
            regex_format_text_value = ?,
            default_range_int_value_from = ?,
            default_range_int_value_to = ?,
            range_int_value_from = ?,
            range_int_value_to = ?,
            min_range_int_value_from = ?,
            min_range_int_value_to = ?,
            max_range_int_value_from = ?,
            max_range_int_value_to = ?,
            is_signed_range_int_value_from = ?,
            is_signed_range_int_value_to = ?,
            default_range_text_value_from = ?,
            default_range_text_value_to = ?,
            range_text_value_from = ?,
            range_text_value_to = ?,
            min_length_range_text_value_from = ?,
            max_length_range_text_value_from = ?,
            min_length_range_text_value_to = ?,
            max_length_range_text_value_to = ?,
            regex_format_range_text_value_from = ?,
            regex_format_range_text_value_to = ?,
            control_title = ?,
            parent_app_setting_id = ?,
            app_setting_type_id = ?,
            app_setting_action_type_id = ?,
            measurement_unit_type_id = ?,
            input_value_type_id = ?,
            modified = CURRENT_TIMESTAMP 
            WHERE id = ?",
            e.code,
            e.name,
            e.description,
            e.disclaimer,
            e.is_enabled,
            e.is_read_on_app_launch,
            e.is_read_on_app_exit,
            e.default_int_value,
            e.int_value,
            e.min_int_value,
            e.max_int_value,
            e.is_signed_int_value,
            e.default_text_value,
            e.text_value,
            e.min_length_text_value,
            e.max_length_text_value,
            e.regex_format_text_value,
            e.default_range_int_value_from,
            e.default_range_int_value_to,
            e.range_int_value_from,
            e.range_int_value_to,
            e.min_range_int_value_from,
            e.min_range_int_value_to,
            e.max_range_int_value_from,
            e.max_range_int_value_to,
            e.is_signed_range_int_value_from,
            e.is_signed_range_int_value_to,
            e.default_range_text_value_from,
            e.default_range_text_value_to,
            e.range_text_value_from,
            e.range_text_value_to,
            e.min_length_range_text_value_from,
            e.max_length_range_text_value_from,
            e.min_length_range_text_value_to,
            e.max_length_range_text_value_to,
            e.regex_format_range_text_value_from,
            e.regex_format_range_text_value_to,
            e.control_title,
            e.parent_app_setting_id,
            e.app_setting_type_id,
            e.app_setting_action_type_id,
            e.measurement_unit_type_id,
            e.input_value_type_id,
            e.id
        )
        .execute(self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM app_setting WHERE id = ?")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
