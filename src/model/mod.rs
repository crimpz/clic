use crate::model::store::{new_db_pool, Db};

mod base;
mod error;
pub mod messages;
pub mod room;
mod store;
pub mod task;
pub mod user;

pub use self::error::{Error, Result};

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    // Constructor
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;

        Ok(ModelManager { db })
    }
    // Returns sqlx db pool reference only for model layer
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
