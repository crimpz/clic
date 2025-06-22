use crate::Ctx;
use crate::model::base;
use crate::model::base::DbBmc;
use crate::model::{ModelManager, Result};
use serde::{Deserialize, Serialize};
use sqlb::Fields;
use sqlx::FromRow;

#[derive(Debug, Clone, Fields, FromRow, Serialize, Deserialize)]
pub struct Room {
    pub id: i64,
    pub room_type: String,
    pub title: String,
}

#[derive(Fields, Deserialize)]
pub struct RoomCreate {
    pub room_type: String,
    pub title: String,
}

#[derive(Fields, Deserialize)]
pub struct RoomUpdate {
    pub title: Option<String>,
}

pub struct RoomBmc;

impl DbBmc for RoomBmc {
    const TABLE: &'static str = "rooms";
}

impl RoomBmc {
    pub async fn create(ctx: &Ctx, mm: &ModelManager, title: RoomCreate) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, title).await
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
        room_update: RoomUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, room_update).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}
