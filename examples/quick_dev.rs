use anyhow::Result;
use chrono::Utc;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    let req_create_user = hc.do_post(
        "/api/create_user",
        json!({
        "username": "Dallas",
        "pwd": "hello"
        }),
    );
    req_create_user.await?.print().await?;

    let req_create_user2 = hc.do_post(
        "/api/create_user",
        json!({
        "username": "test",
        "pwd": "welcome"
        }),
    );
    req_create_user2.await?.print().await?;

    let req_create_user3 = hc.do_post(
        "/api/create_user",
        json!({
        "username": "Friend",
        "pwd": "welcome"
        }),
    );
    req_create_user3.await?.print().await?;

    let req_login = hc.do_post(
        "/api/login",
        json!({
        "username": "Dallas",
        "pwd": "hello"
        }),
    );
    req_login.await?.print().await?;

    let add_friend = hc.do_post(
        "/api/rpc",
        json!({
            "method": "add_friend",
            "params": {
            "data": {
            "name": "test",
        }
        }
        }),
    );
    add_friend.await?.print().await?;

    let add_friend2 = hc.do_post(
        "/api/rpc",
        json!({
            "method": "add_friend",
            "params": {
            "data": {
            "name": "Friend",
        }
        }
        }),
    );
    add_friend2.await?.print().await?;

    let get_friends = hc.do_post(
        "/api/rpc",
        json!({
            "method": "get_friends",
        }),
    );
    get_friends.await?.print().await?;

    let req_create_room = hc.do_post(
        "/api/rpc",
        json!({
               "id": 1,
               "method": "create_room",
               "params": {
               "data": {
               "title": "General Chat"
               }
           }
        }),
    );
    req_create_room.await?.print().await?;

    let req_create_room2 = hc.do_post(
        "/api/rpc",
        json!({
               "id": 1,
               "method": "create_room",
               "params": {
               "data": {
               "title": "Test Chat"
               }
           }
        }),
    );
    req_create_room2.await?.print().await?;

    let req_list_rooms = hc.do_post(
        "/api/rpc",
        json!({
        "id": 1,
        "method": "list_rooms"
        }),
    );
    req_list_rooms.await?.print().await?;

    let req_update_room = hc.do_post(
        "/api/rpc",
        json!({
                "method": "update_room",
                "params": {
                "id": 2,
                "data": {
                "title": "School Chat"
                }
            }
        }),
    );
    req_update_room.await?.print().await?;

    let req_delete_room = hc.do_post(
        "/api/rpc",
        json!({
        "method": "delete_room",
        "params": {
        "id": 3,
        }
        }),
    );
    req_delete_room.await?.print().await?;

    let send_message = hc.do_post(
        "/api/rpc",
        json!({
            "method": "send_message",
            "params": {
            "data": {
               "message_text": "Hello",
               "message_room_id": 1,
               "message_user_name": "Dallas",
            }
        }
        }),
    );
    send_message.await?.print().await?;

    let send_message2 = hc.do_post(
        "/api/rpc",
        json!({
            "method": "send_message",
            "params": {
            "data": {
               "message_text": "Hello",
               "message_room_id": 2,
               "message_user_name": "Dallas",
            }
        }
        }),
    );
    send_message2.await?.print().await?;

    let req_list_messages = hc.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "get_messages_by_room_id",
            "params": 1,
        }),
    );
    req_list_messages.await?.print().await?;

    let send_message3 = hc.do_post(
        "/api/rpc",
        json!({
            "method": "send_message",
            "params": {
            "data": {
               "message_text": "test",
               "message_room_id": 2,
               "message_user_name": "Dallas",
            }
        }
        }),
    );
    send_message3.await?.print().await?;

    let get_recent_message = hc.do_post(
        "/api/rpc",
        json!({
            "method": "get_recent_room_messages_by_id",
            "params": {
               "room_id": 2,
               "message_id": 1,
        }}),
    );
    get_recent_message.await?.print().await?;

    let send_private = hc.do_post(
        "/api/rpc",
        json!({
            "method": "send_private_message",
            "params": {
            "data": {
                "sender_name": "Dallas",
                "receiver_name": "Friend",
                "message_text": "Hi friend!",
            }
        }
        }),
    );
    send_private.await?.print().await?;

    let get_private = hc.do_post(
        "/api/rpc",
        json!({
            "method": "get_private_messages",
            "params": "Friend",
        }),
    );
    get_private.await?.print().await?;

    let req_logoff = hc.do_post(
        "/api/logoff",
        json!({
        "logoff": true
        }),
    );
    //req_logoff.await?.print().await?;

    Ok(())
}
