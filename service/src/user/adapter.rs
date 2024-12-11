use bomboni_request::{
    error::{CommonError, RequestErrorExt},
    parse::RequestParse,
};
use prost::Name;
use std::fmt::Debug;
use tonic::{Request, Response};

use grpc_sky_api::{
    dto::SignUpRequestDto,
    proto::{user_service_server::UserService, GetMeRequest, SignUpRequest, SignUpResponse, User},
};

use crate::{
    context::Context,
    user::{
        repository::UserRepositoryArc,
        sign_up_command::{SignUpCommand, SignUpCommandInput},
    },
};

pub struct UserAdapter {
    user_repository: UserRepositoryArc,
    sign_up_command: SignUpCommand,
}

impl UserAdapter {
    pub fn new(user_repository: UserRepositoryArc, sign_up_command: SignUpCommand) -> Self {
        Self {
            user_repository,
            sign_up_command,
        }
    }
}

#[tonic::async_trait]
impl UserService for UserAdapter {
    #[tracing::instrument(skip(self, request), fields(
        remote_addr = request.remote_addr().map(|addr| addr.to_string()),
    ), err(Debug))]
    async fn sign_up(
        &self,
        request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, tonic::Status> {
        let request = SignUpRequestDto::parse(request.into_inner())?;

        let output = self
            .sign_up_command
            .execute(SignUpCommandInput {
                name: &request.name,
                password: &request.password,
            })
            .await?;

        Ok(Response::new(SignUpResponse {
            user_id: output.user_id.to_string(),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(
        remote_addr = request.remote_addr().map(|addr| addr.to_string()),
    ), err(Debug))]
    async fn get_me(
        &self,
        request: Request<GetMeRequest>,
    ) -> Result<Response<User>, tonic::Status> {
        let context = Context::from_request(&request);
        let user_id = context.authenticate()?;

        let Some(user) = self.user_repository.select(user_id).await? else {
            return Err(CommonError::Unauthorized
                .wrap_request(GetMeRequest::NAME)
                .into());
        };

        Ok(Response::new(User {
            id: user.id.to_string(),
            name: user.name,
        }))
    }
}

impl Debug for UserAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserAdapter").finish()
    }
}
