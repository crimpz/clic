use crate::ctx::Ctx;
use crate::log::log_request;
use crate::web::{error::Error, rpc::RpcInfo};
use axum::{
    Json,
    body::Body,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::{json, to_value};
use tracing::debug;
use uuid::Uuid;

pub async fn mw_response_map(req: Request<Body>, next: Next) -> Response {
    debug!("{:<12} - mw_response_map", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    let uri = req.uri().clone();
    let method = req.method().clone();

    let mut res = next.run(req).await;

    let rpc_info = res.extensions().get::<RpcInfo>().cloned();
    let ctx = res.extensions().get::<Ctx>().cloned();
    let web_error = res.extensions().get::<Error>().cloned();
    let client_status_error = web_error.as_ref().map(|se| se.client_status_and_error());

    if let Some((status_code, client_error)) = &client_status_error {
        let client_error_value = to_value(client_error).ok();
        let message = client_error_value.as_ref().and_then(|v| v.get("message"));
        let detail = client_error_value.as_ref().and_then(|v| v.get("detail"));

        let body = json!({
            "error": {
                "message": message,
                "data": {
                    "req_uuid": uuid.to_string(),
                    "detail": detail
                },
            }
        });

        debug!("client_error_body: {body}");

        res = (*status_code, Json(body)).into_response();
    }

    let client_error = client_status_error.unzip().1;

    let _ = log_request(
        uuid,
        method,
        uri,
        rpc_info.as_ref(),
        ctx,
        web_error.as_ref(),
        client_error,
    )
    .await;

    debug!("\n");
    res
}
