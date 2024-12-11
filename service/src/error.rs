use bomboni_request::error::{CommonError, RequestError};
use thiserror::Error;
use tonic::{transport, Code, Status};
use tracing::error;

use grpc_sky_api::error::UserError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("request error: {0}")]
    Request(#[from] RequestError),
    #[error("status error: {0}")]
    Status(#[from] Status),
}

pub type AppResult<T> = Result<T, AppError>;

macro_rules! impl_internal_errors {
    ( $( $type:ty ),* $(,)? ) => {
        $(
        impl From<$type> for AppError {
            fn from(err: $type) -> Self {
                AppError::Internal(Box::new(err))
            }
        }
        )*
    };
}
impl_internal_errors![config::ConfigError, transport::Error, mysql_async::Error];

macro_rules! impl_request_errors {
    ( $( $type:ty ),* $(,)? ) => {
        $(
        impl From<$type> for AppError {
            fn from(err: $type) -> Self {
                RequestError::from(err).into()
            }
        }
        )*
    };
}
impl_request_errors!(UserError, CommonError);

impl From<AppError> for Status {
    fn from(err: AppError) -> Self {
        match err {
            AppError::Request(err) => err.into(),
            AppError::Status(status) => status,
            _ => {
                error!("internal service error: {}", err);
                Status::internal(Code::Internal.description())
            }
        }
    }
}
