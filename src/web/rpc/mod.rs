use crate::{
    ctx::Ctx,
    model::user::*,
    model::ModelManager,
    web::{
        rpc::message_rpc::{
            get_messages_by_room_id, get_private_messages, get_recent_room_messages_by_id,
            send_message, send_private_message,
        },
        rpc::room_rpc::{create_room, delete_room, list_rooms, update_room},
        Error, Result,
    },
};
use axum::{extract::State, Json};
use axum::{
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use log::debug;
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};

mod message_rpc;
mod room_rpc;
mod task_rpc;

#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    id: i64,
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsIded {
    id: i64,
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}

async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    let rpc_info = RpcInfo {
        id: rpc_req.id.clone(),
        method: rpc_req.method.clone(),
    };

    let mut res = _rpc_handler(ctx, mm, rpc_req).await.into_response();
    res.extensions_mut().insert(rpc_info);

    res
}

#[derive(Debug)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}

macro_rules! exec_rpc_fn {
    // With params
    ($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
        let rpc_fn_name = stringify!($rpc_fn);
        let params = $rpc_params.ok_or(Error::RpcMissingParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;
        let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;

        $rpc_fn($ctx, $mm, params).await.map(to_value)??
    }};

    // Without params
    ($rpc_fn:expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    };
}
async fn _rpc_handler(ctx: Ctx, mm: ModelManager, rpc_req: RpcRequest) -> Result<Json<Value>> {
    let RpcRequest {
        id: rpc_id,
        method: rpc_method,
        params: rpc_params,
    } = rpc_req;
    debug!("{:12} - _rpc_handler - method: {rpc_method}", "HANDLER");

    let result_json: Value = match rpc_method.as_str() {
        // Room RPC methods
        "create_room" => exec_rpc_fn!(create_room, ctx, mm, rpc_params),
        "list_rooms" => exec_rpc_fn!(list_rooms, ctx, mm),
        "update_room" => exec_rpc_fn!(update_room, ctx, mm, rpc_params),
        "delete_room" => exec_rpc_fn!(delete_room, ctx, mm, rpc_params),

        // Message RPC methods
        "get_messages_by_room_id" => exec_rpc_fn!(get_messages_by_room_id, ctx, mm, rpc_params),
        "get_recent_room_messages_by_id" => {
            exec_rpc_fn!(get_recent_room_messages_by_id, ctx, mm, rpc_params)
        }
        "send_message" => exec_rpc_fn!(send_message, ctx, mm, rpc_params),
        "send_private_message" => exec_rpc_fn!(send_private_message, ctx, mm, rpc_params),
        "get_private_messages" => exec_rpc_fn!(get_private_messages, ctx, mm, rpc_params),

        // User RPC methods
        "add_friend" => exec_rpc_fn!(UserBmc::add_friend, ctx, mm, rpc_params),
        "get_friends" => exec_rpc_fn!(UserBmc::get_friends, ctx, mm),

        // Fallback as Err
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    let body_response = json!({
    "id": rpc_id,
    "result": result_json
    });

    Ok(Json(body_response))
}
