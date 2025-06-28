use crate::mw_ctx_require;
use crate::{
    ctx::Ctx,
    model::ModelManager,
    model::user::*,
    web::{
        error::{Error, Result},
        rpc::message::{
            get_messages_by_room_id, get_private_messages, send_message, send_private_message,
        },
        rpc::room::{create_room, delete_room, list_rooms, update_room},
        rpc::voice::join_voice,
    },
};

use axum::{Json, extract::State};
use axum::{
    Router,
    middleware::from_fn,
    response::{IntoResponse, Response},
    routing::post,
};
use log::debug;
use serde::Deserialize;
use serde_json::{Value, from_value, json, to_value};

mod message;
mod room;
mod voice;

#[derive(Deserialize)]
struct RpcRequest {
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
        .route_layer(from_fn(mw_ctx_require))
}

async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    let rpc_info = RpcInfo {
        method: rpc_req.method.clone(),
    };

    let mut res = _rpc_handler(ctx, mm, rpc_req).await.into_response();
    res.extensions_mut().insert(rpc_info);

    res
}

#[derive(Clone, Debug)]
pub struct RpcInfo {
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
        method: rpc_method,
        params: rpc_params,
    } = rpc_req;
    debug!("{:12} - rpc_handler - method: {rpc_method}", "HANDLER");

    let result_json: Value = match rpc_method.as_str() {
        // Voice RPC methods
        //"get_audio_room" => exec_rpc_fn!(get_audio_room_info, ctx, mm, rpc_params),
        "join_voice" => exec_rpc_fn!(join_voice, ctx, mm, rpc_params),

        // Room RPC methods
        "create_room" => exec_rpc_fn!(create_room, ctx, mm, rpc_params),
        "list_rooms" => exec_rpc_fn!(list_rooms, ctx, mm),
        "update_room" => exec_rpc_fn!(update_room, ctx, mm, rpc_params),
        "delete_room" => exec_rpc_fn!(delete_room, ctx, mm, rpc_params),

        // Message RPC methods
        "get_messages_by_room_id" => exec_rpc_fn!(get_messages_by_room_id, ctx, mm, rpc_params),
        "send_message" => exec_rpc_fn!(send_message, ctx, mm, rpc_params),
        "send_private_message" => exec_rpc_fn!(send_private_message, ctx, mm, rpc_params),
        "get_private_messages" => exec_rpc_fn!(get_private_messages, ctx, mm, rpc_params),

        // User RPC methods
        "add_friend" => exec_rpc_fn!(UserBmc::add_friend, ctx, mm, rpc_params),
        "get_friends" => exec_rpc_fn!(UserBmc::get_friends, ctx, mm),
        "find_by_id" => exec_rpc_fn!(UserBmc::find_username_by_id, ctx, mm, rpc_params),

        // Fallback as Err
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    let body_response = json!({
    "result": result_json
    });

    Ok(Json(body_response))
}
