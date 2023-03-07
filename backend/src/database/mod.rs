use sqlx::{PgPool, postgres::PgPoolOptions};

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

}
