use crate::model::base;
use crate::model::base::DbBmc;
use crate::model::{Error, ModelManager, Result};
use crate::Ctx;
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::FromRow;

#[derive(Debug, Clone, Fields, FromRow, Serialize, Deserialize)]
pub struct Room {
    pub id: i64,
    pub title: String,
}

#[derive(Fields, Deserialize)]
pub struct RoomForCreate {
    pub title: String,
}

#[derive(Fields, Deserialize)]
pub struct RoomForUpdate {
    pub title: Option<String>,
}

pub struct RoomBmc;

impl DbBmc for RoomBmc {
    const TABLE: &'static str = "rooms";
}

impl RoomBmc {
    pub async fn create(ctx: &Ctx, mm: &ModelManager, task_c: RoomForCreate) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, task_c).await
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Room> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Room>> {
        base::list::<Self, _>(ctx, mm).await
    }

    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        task_u: RoomForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, task_u).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}
