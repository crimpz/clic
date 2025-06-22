use crate::Ctx;
use crate::model::ModelManager;
use crate::model::user::UserBmc;
use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::typed_header::TypedHeader;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::unbounded_channel;

pub async fn ws_handler(
    ctx: Ctx,
    ws: WebSocketUpgrade,
    TypedHeader(_cookies): TypedHeader<headers::Cookie>,
    State(mm): State<ModelManager>,
) -> impl IntoResponse {
    let user_id = ctx.user_id();

    match UserBmc::find_username_by_id(ctx, mm.clone(), user_id).await {
        Ok(Some(username_only)) => {
            let username = username_only.username;
            Ok(ws.on_upgrade(move |socket| handle_socket(socket, username, mm))) as Result<_, _>
        }
        Ok(None) => Err((StatusCode::UNAUTHORIZED, "User not found")),
        Err(e) => {
            tracing::error!("DB error in ws_handler: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"))
        }
    }
}

async fn handle_socket(socket: WebSocket, user_id: String, mm: ModelManager) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = unbounded_channel::<Message>();

    mm.ws_broadcast.register_user(user_id.clone(), tx).await;

    // Forward messages from channel to the socket
    let user_id_clone = user_id.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                eprintln!("Failed to send message to user {}", user_id_clone);
                break;
            }
        }
    });

    // Read messages from the socket
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                println!("[{}] Received: {}", user_id, text);
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    mm.ws_broadcast.unregister_user(&user_id).await;
}
