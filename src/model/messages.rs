use crate::Ctx;
use crate::model::base;
use crate::model::base::DbBmc;
use crate::model::user::{User, UserBmc};
use crate::model::{ModelManager, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::{FromRow, Row};
type UtcDateTime = DateTime<Utc>;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Image {
    pub id: uuid::Uuid,
    pub message_id: i64,
    pub user_id: i64,
    pub filename: String,
    pub content_type: String,
    pub storage_path: String,
    pub uploaded_at: UtcDateTime,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageWithImages {
    pub message_id: i64,
    pub message_text: String,
    pub message_room_id: i64,
    pub message_user_id: i64,
    pub message_datetime: UtcDateTime,
    pub images: Vec<Image>,
}

#[derive(Debug, Clone, Fields, Deserialize, FromRow, Serialize)]
pub struct Message {
    pub message_text: String,
    pub message_room_id: i64,
    pub message_user_id: i64,
}

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct MessageReceived {
    pub message_id: i64,
    pub message_text: String,
    pub message_room_id: i64,
    pub message_user_id: i64,
    //pub message_datetime: UtcDateTime,
}

#[derive(Fields, Serialize, Deserialize, FromRow)]
pub struct FriendMessage {
    pub sender_name: String,
    pub receiver_name: String,
    pub message_text: String,
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
    pub async fn send_message(ctx: &Ctx, mm: &ModelManager, message: Message) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, message).await
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
        _ctx: &Ctx,
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

    pub async fn list_with_images_by_room_id(
        _ctx: &Ctx,
        mm: &ModelManager,
        room_id: i64,
    ) -> Result<Vec<MessageWithImages>> {
        let query = r#"
        SELECT
            m.id AS message_id,
            m.message_text,
            m.message_room_id,
            m.message_user_id,
            m.message_datetime,
            i.id AS image_id,
            i.message_id AS image_message_id,
            i.user_id AS image_user_id,
            i.filename,
            i.content_type,
            i.storage_path,
            i.uploaded_at
        FROM messages m
        LEFT JOIN images i ON m.id = i.message_id
        WHERE m.message_room_id = $1
        ORDER BY m.message_datetime ASC, i.uploaded_at ASC, m.id ASC
    "#;

        let rows = sqlx::query(query).bind(room_id).fetch_all(mm.db()).await?;

        use std::collections::HashMap;

        let mut map: HashMap<i64, MessageWithImages> = HashMap::new();

        for row in rows {
            let msg_id: i64 = row.get("message_id");

            let entry = map.entry(msg_id).or_insert_with(|| MessageWithImages {
                message_id: msg_id,
                message_text: row.get("message_text"),
                message_room_id: row.get("message_room_id"),
                message_user_id: row.get("message_user_id"),
                message_datetime: row.get("message_datetime"),
                images: vec![],
            });

            if let Some(image_id) = row.try_get::<uuid::Uuid, _>("image_id").ok() {
                entry.images.push(Image {
                    id: image_id,
                    message_id: row.get("image_message_id"),
                    user_id: row.get("image_user_id"),
                    filename: row.get("filename"),
                    content_type: row.get("content_type"),
                    storage_path: row.get("storage_path"),
                    uploaded_at: row.get("uploaded_at"),
                });
            }
        }

        Ok(map.into_values().collect())
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
            .order_by("message_datetime")
            .fetch_all(mm.db())
            .await?;

        let mut messages_from: Vec<FriendMessage> = sqlb::select()
            .table("private_messages")
            .columns(FriendMessage::field_names())
            .and_where("sender_name", "=", from)
            .and_where("receiver_name", "=", to)
            .order_by("message_datetime")
            .fetch_all(mm.db())
            .await?;

        // Combine messages
        let mut combined_messages = Vec::with_capacity(messages_to.len() + messages_from.len());
        combined_messages.append(&mut messages_to);
        combined_messages.append(&mut messages_from);

        Ok(combined_messages)
    }
}
