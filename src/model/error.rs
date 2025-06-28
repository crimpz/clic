use crate::crypt;
use crate::model::store;
use axum::body::Body;
use axum::response::Response;
use hyper::StatusCode;
use serde::Serialize;
use serde_with::{DisplayFromStr, serde_as};
use std::sync::Arc;
pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Clone, Debug, Serialize)]
pub enum Error {
    // -- Modules
    EntityNotFound {
        entity: &'static str,
        id: i64,
    },
    Store(store::Error),
    TicketDeleteFailIdNotFound {
        id: u64,
    },
    Crypt(crypt::Error),

    // -- Externals
    Sqlx(#[serde_as(as = "DisplayFromStr")] Arc<sqlx::Error>),

    // -- HTTP bridge
    Http {
        #[serde(skip_serializing)]
        status: StatusCode,
        message: &'static str,
    },
}
impl From<(Response<Body>, &'static str)> for Error {
    fn from((res, msg): (Response<Body>, &'static str)) -> Self {
        Self::Http {
            status: res.status(),
            message: msg,
        }
    }
}

impl From<(StatusCode, &'static str)> for Error {
    fn from((status, msg): (StatusCode, &'static str)) -> Self {
        Self::Http {
            status,
            message: msg,
        }
    }
}

impl From<crypt::Error> for Error {
    fn from(val: crypt::Error) -> Self {
        Self::Crypt(val)
    }
}

impl From<sqlx::Error> for Error {
    fn from(val: sqlx::Error) -> Self {
        Self::Sqlx(Arc::new(val))
    }
}

impl From<store::Error> for Error {
    fn from(val: store::Error) -> Self {
        Self::Store(val)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
