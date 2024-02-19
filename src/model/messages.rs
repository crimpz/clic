use crate::model::base;
use crate::model::base::DbBmc;
use crate::model::user::{User, UserBmc};
use crate::model::{ModelManager, Result};
use crate::Ctx;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::FromRow;

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Message {
    pub id: i64,
    // need to find a way to get message_datetime: DateTime<Utc> to work here
    //pub message_datetime: DateTime<Utc>,
    pub message_text: String,
    pub message_room_id: i64,
    pub message_user_name: String,
}

#[derive(Fields, Serialize, Deserialize, FromRow)]
pub struct FriendMessage {
    pub id: i64,
    pub sender_name: String,
    pub receiver_name: String,
    pub message_text: String,
}

#[derive(Fields, Deserialize)]
pub struct MessageToRoom {
    pub message_text: String,
    pub message_room_id: i64,
    pub message_user_name: String,
}

#[derive(Fields, Deserialize)]
pub struct RecentMessage {
    pub room_id: i64,
    pub message_id: i64,
}

#[derive(Fields, Serialize, Deserialize, FromRow)]
pub struct MessageToFriend {
    pub sender_name: String,
    pub receiver_name: String,
    pub message_text: String,
}

pub struct MessageBmc;

impl DbBmc for MessageBmc {
    const TABLE: &'static str = "messages";
}

impl MessageBmc {
    pub async fn send_message(ctx: &Ctx, mm: &ModelManager, task_c: MessageToRoom) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, task_c).await
    }

    pub async fn send_private_message(
        ctx: &Ctx,
        mm: &ModelManager,
        task_c: MessageToFriend,
    ) -> Result<i64> {
        base::create_private::<Self, _>(ctx, mm, task_c).await
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Message> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn list_by_room_id(
        ctx: &Ctx,
        mm: &ModelManager,
        room_id: i64,
    ) -> Result<Vec<Message>> {
        let rooms: Vec<Message> = sqlb::select()
            .table(Self::TABLE)
            .columns(Message::field_names())
            .and_where("message_room_id", "=", room_id)
            .order_by("id")
            .fetch_all(mm.db())
            .await?;

        Ok(rooms)
    }

    pub async fn get_recent_room_messages_by_id(
        ctx: &Ctx,
        mm: &ModelManager,
        task_u: RecentMessage,
    ) -> Result<Vec<Message>> {
        let rooms: Vec<Message> = sqlb::select()
            .table(Self::TABLE)
            .columns(Message::field_names())
            .and_where("message_room_id", "=", task_u.room_id)
            .and_where("id", ">", task_u.message_id)
            .order_by("id")
            .fetch_all(mm.db())
            .await?;

        Ok(rooms)
    }

    pub async fn get_private_messages(
        ctx: &Ctx,
        mm: &ModelManager,
        receiver: String,
    ) -> Result<Vec<FriendMessage>> {
        let user: User = UserBmc::get(&ctx, &mm, ctx.user_id()).await?;
        let from = receiver.clone();
        let to = user.username.clone();

        let mut messages_to: Vec<FriendMessage> = sqlb::select()
            .table("private_messages")
            .columns(FriendMessage::field_names())
            .and_where("sender_name", "=", user.username)
            .and_where("receiver_name", "=", receiver)
            .order_by("id")
            .fetch_all(mm.db())
            .await?;

        let mut messages_from: Vec<FriendMessage> = sqlb::select()
            .table("private_messages")
            .columns(FriendMessage::field_names())
            .and_where("sender_name", "=", from)
            .and_where("receiver_name", "=", to)
            .order_by("id")
            .fetch_all(mm.db())
            .await?;

        // Combine messages_to and messages_from
        let mut combined_messages = Vec::new();
        combined_messages.append(&mut messages_to);
        combined_messages.append(&mut messages_from);

        // Sort the combined messages based on the 'id' field
        combined_messages.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(combined_messages)
    }
}
