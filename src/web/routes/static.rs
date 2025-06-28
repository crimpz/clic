use crate::config;
use axum::{Router, http::StatusCode, response::IntoResponse};
use tower::service_fn;
use tower_http::services::ServeDir;

pub fn serve_dir() -> Router {
    let web_root = &config().WEB_FOLDER;

    let handle_404 = service_fn(|_req| async {
        let response = (StatusCode::NOT_FOUND, "Resource not found").into_response();
        Ok::<_, std::convert::Infallible>(response)
    });

    Router::new().fallback_service(ServeDir::new(web_root.clone()).not_found_service(handle_404))
}
