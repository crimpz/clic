use crate::model::base;
use crate::model::base::DbBmc;
use crate::model::messages::{FriendMessage, Message, MessageToFriend, MessageWithImages};
use crate::model::room::{Room, RoomBmc};
use crate::model::{ModelManager, Result};
use crate::{
    ctx::Ctx,
    model::user::{User, UserBmc},
};
use axum::http::StatusCode;
use sqlb::Fields;
use sqlx::FromRow;

#[derive(Debug, FromRow, Fields)]
pub struct RoomParticipant {
    pub room_id: i64,
    pub user_id: i64,
}

pub struct ChatUsersBmc;
impl DbBmc for ChatUsersBmc {
    const TABLE: &'static str = "room_participants";
}

#[derive(serde::Deserialize)]
pub struct ParamsJoinVoice {
    pub room_id: i64,
}

#[derive(serde::Serialize)]
pub struct JoinVoiceResult {
    pub room: Room,
    pub users: Vec<i64>,
}

impl ChatUsersBmc {
    pub async fn insert(ctx: &Ctx, mm: &ModelManager, row: RoomParticipant) -> Result<()> {
        base::create::<Self, _>(ctx, mm, row).await.map(|_| ())
    }

    pub async fn list_by_room(
        ctx: &Ctx,
        mm: &ModelManager,
        room_id: i64,
    ) -> Result<Vec<RoomParticipant>> {
        let db = mm.db();
        let items = sqlb::select()
            .table(Self::TABLE)
            .columns(&["user_id"])
            .and_where("room_id", "=", room_id)
            .order_by("joined_at")
            .fetch_all::<_, RoomParticipant>(db)
            .await?;
        Ok(items)
    }
}

pub async fn join_voice(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsJoinVoice,
) -> Result<JoinVoiceResult> {
    let ParamsJoinVoice { room_id } = params;

    let room = RoomBmc::get(&ctx, &mm, room_id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Room not found"))?;

    if room.room_type != "voice" {
        return Err((StatusCode::BAD_REQUEST, "Not a voice room").into());
    }

    let _ = ChatUsersBmc::insert(
        &ctx,
        &mm,
        RoomParticipant {
            room_id,
            user_id: ctx.user_id(),
        },
    )
    .await;

    let users = ChatUsersBmc::list_by_room(&ctx, &mm, room_id)
        .await?
        .into_iter()
        .map(|u| u.user_id)
        .collect::<Vec<_>>();

    let user: User = UserBmc::get(&ctx, &mm, ctx.user_id()).await?;
    mm.ws_broadcast
        .broadcast_voice(room_id, user.id, &user.username)
        .await;

    Ok(JoinVoiceResult { room: room, users })
}
