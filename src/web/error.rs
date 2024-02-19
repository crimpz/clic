use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tracing::debug;

use crate::{crypt, model, web};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    // RPC
    RpcMethodUnknown(String),
    RpcMissingParams { rpc_method: String },
    RpcFailJsonParams { rpc_method: String },

    LoginFailUsernameNotFound,
    LoginFailUserHasNoPwd { user_id: i64 },
    LoginFailPwdNotMatching { user_id: i64 },

    FailedToGetMessageByRoomId,
    //CtxExtError
    CtxExt(web::mw_auth::CtxExtError),

    //Model Errors
    Model(model::Error),
    Crypt(crypt::Error),

    // External Modules
    SerdeJson(String),

    //Config Errors
    ConfigMissingEnv(&'static str),
}

impl From<model::Error> for Error {
    fn from(val: model::Error) -> Self {
        Self::Model(val)
    }
}

impl From<crypt::Error> for Error {
    fn from(val: crypt::Error) -> Self {
        Self::Crypt(val)
    }
}

impl From<serde_json::Error> for Error {
    fn from(val: serde_json::Error) -> Self {
        Self::SerdeJson(val.to_string())
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use web::Error::*;
        #[allow(unreachable_patterns)]
        match self {
            // Login
            LoginFailUsernameNotFound
            | LoginFailPwdNotMatching { .. }
            | LoginFailUserHasNoPwd { .. } => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            // Auth
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // Model
            Model(model::Error::EntityNotFound { entity, id }) => (
                StatusCode::BAD_REQUEST,
                ClientError::ENTITY_NOT_FOUND { entity, id: *id },
            ),

            // Fallback
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
    SERVICE_ERROR,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!("{:<12} - {self:?}", "INTO_RES");

        // Create a placeholder Axum response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the error into response.
        response.extensions_mut().insert(self);

        response
    }
}
