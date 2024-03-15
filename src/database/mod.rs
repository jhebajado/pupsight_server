use std::{sync::RwLock, time::Duration};

use sea_orm::{ConnectOptions, Database as SeaDatabase};

pub(crate) struct Database {
    connection: RwLock<sea_orm::DatabaseConnection>,
}

impl Database {
    #[inline]
    pub(crate) async fn new(url: &str) -> Self {
        let mut options = ConnectOptions::new(url);
        options
            .max_connections(8)
            .min_connections(4)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .set_schema_search_path("public");

        let connection = RwLock::new(SeaDatabase::connect(options).await.unwrap());

        Self { connection }
    }
}
