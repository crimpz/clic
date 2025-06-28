use super::ParamsForCreate;
use crate::model::ModelManager;
use crate::model::Result;
use crate::model::WsEvent;
use crate::model::messages::{FriendMessage, Message, MessageToFriend, MessageWithImages};
use crate::model::user::{User, UserBmc};
use crate::{ctx::Ctx, model::messages::MessageBmc};

#[derive(serde::Serialize)]
pub struct MessageResponse {
    id: i64,
}

pub async fn send_message(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForCreate<Message>,
) -> Result<MessageResponse> {
    let ParamsForCreate { data } = params;
    let message = MessageBmc::send_message(&ctx, &mm, data.clone()).await?;
    let user: User = UserBmc::get(&ctx, &mm, ctx.user_id()).await?;
    let username = user.username.clone();

    let msg = WsEvent::NewRoomMessage {
        room_id: data.message_room_id,
        from: username.clone(),
        content: data.message_text.clone(),
    };

    let text = serde_json::to_string(&msg);

    tracing::debug!(
        "Sending websocket message: username = {}, message = {}",
        &username,
        &data.message_text
    );

    mm.ws_broadcast
        .broadcast_to_room(&username, &data.message_text)
        .await;

    Ok(MessageResponse { id: message })
}

pub async fn send_private_message(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForCreate<MessageToFriend>,
) -> Result<i64> {
    let ParamsForCreate { data } = params;
    let message = MessageBmc::send_private_message(&ctx, &mm, data).await?;

    Ok(message)
}

pub async fn get_messages_by_room_id(
    ctx: Ctx,
    mm: ModelManager,
    params: i64,
) -> Result<Vec<MessageWithImages>> {
    let messages = MessageBmc::list_with_images_by_room_id(&ctx, &mm, params).await?;

    Ok(messages)
}

pub async fn get_private_messages(
    ctx: Ctx,
    mm: ModelManager,
    params: String,
) -> Result<Vec<FriendMessage>> {
    let messages = MessageBmc::get_private_messages(&ctx, &mm, params).await?;

    Ok(messages)
}
