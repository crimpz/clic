use super::ParamsForCreate;
use crate::model::messages::{
    FriendMessage, Message, MessageToFriend, MessageToRoom, RecentMessage,
};
use crate::model::ModelManager;
use crate::model::Result;
use crate::{ctx::Ctx, model::messages::MessageBmc};

pub async fn send_message(
    ctx: Ctx,
    mm: ModelManager,
    params: ParamsForCreate<MessageToRoom>,
) -> Result<i64> {
    let ParamsForCreate { data } = params;
    let message = MessageBmc::send_message(&ctx, &mm, data).await?;

    Ok(message)
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
) -> Result<Vec<Message>> {
    let messages = MessageBmc::list_by_room_id(&ctx, &mm, params).await?;

    Ok(messages)
}

pub async fn get_recent_room_messages_by_id(
    ctx: Ctx,
    mm: ModelManager,
    params: RecentMessage,
) -> Result<Vec<Message>> {
    let messages = MessageBmc::get_recent_room_messages_by_id(&ctx, &mm, params).await?;

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
