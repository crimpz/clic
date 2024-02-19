use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::web::mw_auth::{mw_ctx_require, mw_ctx_resolve};
use crate::web::mw_res_map::mw_response_map;
use crate::web::routes_static;
use crate::web::rpc;
use axum::{middleware, Router};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

// Import custom modules
mod config;
mod crypt;
mod ctx;
mod error;
mod log;
mod model;
mod utils;
mod web;

pub mod _dev_utils;
pub use self::error::{Error, Result};
pub use config::config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // -- For Dev Only
    _dev_utils::init_dev().await;

    let mm = ModelManager::new().await?;
    let routes_rpc = rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(web::routes_login::routes(mm.clone()))
        .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_response_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    // Start server
    // Address that server will bind to.
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("{:<12} - {addr}\n", "LISTENING");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}
