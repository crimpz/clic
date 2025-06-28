use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::web::middleware::auth::{mw_ctx_require, mw_ctx_resolve};
use crate::web::middleware::res_map::mw_response_map;
use crate::web::routes::{login::routes, r#static};
use crate::web::rpc;
use crate::web::upload_images::upload_image;
use crate::web::websockets::ws_handler;
use axum::routing::get_service;
use axum::{
    Router,
    http::{HeaderValue, Method},
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    serve,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
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
pub mod web;

pub mod _dev_utils;
pub use self::error::{Error, Result};
pub use config::config;

#[derive(Clone)]
pub struct AppState {
    pub mm: ModelManager,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    println!("Current working dir: {:?}", std::env::current_dir());

    // -- For Dev Only
    _dev_utils::init_dev().await;

    let mm = ModelManager::new().await?;
    let state = AppState {
        mm: mm.clone().into(),
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let login_routes = routes(state.mm.clone());

    let routes_rpc = rpc::routes(state.mm.clone()).layer(
        ServiceBuilder::new()
            .layer(from_fn_with_state(state.mm.clone(), mw_ctx_require))
            .layer(CookieManagerLayer::new()),
    );

    let images = Router::new()
        .nest_service("/uploads", get_service(ServeDir::new("uploads")))
        .with_state(state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn_with_state(state.mm.clone(), mw_ctx_resolve))
                .layer(CookieManagerLayer::new()),
        );

    let image_uploads = Router::new()
        .route("/upload_image", post(upload_image))
        .with_state(state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn_with_state(state.mm.clone(), mw_ctx_resolve))
                .layer(CookieManagerLayer::new()),
        );

    let ws_route = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.mm.clone())
        .layer(
            ServiceBuilder::new()
                .layer(from_fn_with_state(state.mm.clone(), mw_ctx_resolve))
                .layer(CookieManagerLayer::new()),
        );

    let routes_all = Router::new()
        .merge(login_routes)
        .nest("/api", routes_rpc)
        .merge(ws_route)
        .merge(image_uploads)
        .merge(images)
        .layer(from_fn(mw_response_map))
        .layer(from_fn_with_state(state.mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .fallback_service(r#static::serve_dir());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("{:<12} - Listening on http://{}", "LISTENING", addr);

    let listener = TcpListener::bind(addr).await;
    let _ = serve(listener.expect("REASON"), routes_all).await;

    Ok(())
}
