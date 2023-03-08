use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::configuration::DatabaseSettings;

pub struct YogaDatabase {
    pool: PgPool,
}

impl YogaDatabase {
    pub fn new(settings: DatabaseSettings) -> Self {
        let pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(settings.with_db());
        Self { pool }
    }

    pub async fn get_user_id(&self, email: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT user_id FROM user_profile WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
        Ok(result.map(|r| r.user_id))
    }
}
