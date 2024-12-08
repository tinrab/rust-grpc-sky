use bomboni_request::parse::RequestParse;
use tonic::Request;
use tracing::debug;

use grpc_sky_api::{
    dto::SignUpRequestDto,
    error::UserError,
    proto::{
        user_error::UserErrorReason, user_service_server::UserService, SignUpRequest,
        SignUpResponse,
    },
};

use crate::error::AppResult;

#[derive(Debug)]
pub struct UserAdapter;

impl UserAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl UserService for UserAdapter {
    #[tracing::instrument]
    async fn sign_up(
        &self,
        request: Request<SignUpRequest>,
    ) -> Result<tonic::Response<SignUpResponse>, tonic::Status> {
        debug!("request from {:?}", request.remote_addr());

        let request = SignUpRequestDto::parse(request.into_inner())?;

        AppResult::Err(UserError::new(UserErrorReason::IncorrectCredentials).into())?;

        unreachable!()
    }
}
