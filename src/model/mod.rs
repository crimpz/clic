use crate::model::store::{Db, new_db_pool};
use axum::extract::ws::Message;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc::UnboundedSender};
mod base;
mod error;
mod store;

pub mod messages;
pub mod room;
pub mod user;
pub use self::error::{Error, Result};

#[derive(Clone)]
pub struct WsManager {
    users: Arc<RwLock<HashMap<String, UnboundedSender<Message>>>>,
}

#[derive(Serialize)]
pub enum WsEvent {
    NewRoomMessage {
        alert_type: String,
        room_id: i64,
        from: String,
        content: String,
    },
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_user(&self, user_id: String, tx: UnboundedSender<Message>) {
        self.users.write().await.insert(user_id, tx);
    }

    pub async fn unregister_user(&self, user_id: &String) {
        self.users.write().await.remove(user_id);
    }

    pub async fn broadcast_to_user(&self, user_id: &String, msg: &str) {
        let users = self.users.read().await;
        if let Some(tx) = users.get(user_id) {
            // Ignore send errors (e.g., if receiver is dropped)
            let _ = tx.send(Message::Text(msg.to_string()));
        }
    }

    pub async fn broadcast_to_room(&self, user_id: &String, msg: &str) {
        let users = self.users.read().await;

        if let Some(tx) = users.get(user_id) {
            let _ = tx.send(Message::Text(msg.to_string()));
        }
    }
}

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
    pub ws_broadcast: WsManager,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let db = new_db_pool().await?;
        Ok(ModelManager {
            db,
            ws_broadcast: WsManager::new(),
        })
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}
