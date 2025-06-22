use crate::model::ModelManager;
use crate::model::room::{Room, RoomCreate, RoomUpdate};
use crate::web::Result;
use crate::{ctx::Ctx, model::room::RoomBmc};

use super::{ParamsForCreate, ParamsForUpdate, ParamsIded};

pub async fn create_room(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForCreate<RoomCreate>,
) -> Result<Room> {
    let ParamsForCreate { data } = params;
    let room = RoomBmc::create(&ctx, &mm, data).await?;
    let id = RoomBmc::get(&ctx, &mm, room).await?;
    Ok(id)
}

pub async fn list_rooms(ctx: Ctx, mm: ModelManager) -> Result<Vec<Room>> {
    let rooms = RoomBmc::list(&ctx, &mm).await?;
    Ok(rooms)
}

pub async fn update_room(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForUpdate<RoomUpdate>,
) -> Result<Room> {
    let ParamsForUpdate { id, data } = params;
    RoomBmc::update(&ctx, &mm, id, data).await?;
    let room = RoomBmc::get(&ctx, &mm, id).await?;
    Ok(room)
}

pub async fn delete_room(ctx: Ctx, mm: ModelManager, params: ParamsIded) -> Result<Room> {
    let ParamsIded { id } = params;
    let room = RoomBmc::get(&ctx, &mm, id).await?;
    RoomBmc::delete(&ctx, &mm, id).await?;
    Ok(room)
}
