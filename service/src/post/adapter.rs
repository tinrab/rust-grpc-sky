use bomboni_request::parse::RequestParse;
use std::fmt::Debug;
use tonic::{Request, Response};

use grpc_sky_api::{
    dto::{ListPostsRequestDto, PostRequestDto},
    proto::{
        post_service_server::PostService, ListPostsRequest, ListPostsResponse, PostRequest,
        PostResponse,
    },
};

use crate::{
    context::Context,
    post::{
        create_command::{CreatePostCommand, CreatePostCommandInput},
        query_manager::PostQueryManager,
    },
};

pub struct PostAdapter {
    create_post_command: CreatePostCommand,
    post_query_manager: PostQueryManager,
}

impl PostAdapter {
    pub fn new(
        create_post_command: CreatePostCommand,
        post_query_manager: PostQueryManager,
    ) -> Self {
        Self {
            create_post_command,
            post_query_manager,
        }
    }
}

#[tonic::async_trait]
impl PostService for PostAdapter {
    #[tracing::instrument(skip(self, request), fields(
        remote_addr = request.remote_addr().map(|addr| addr.to_string()),
    ), err(Debug))]
    async fn post(
        &self,
        request: Request<PostRequest>,
    ) -> Result<Response<PostResponse>, tonic::Status> {
        let context = Context::from_request(&request);

        let request = PostRequestDto::parse(request.into_inner())?;

        let output = self
            .create_post_command
            .execute(
                &context,
                CreatePostCommandInput {
                    content: &request.content,
                },
            )
            .await?;

        Ok(Response::new(PostResponse {
            post_id: output.post_id.to_string(),
            create_time: Some(output.create_time.into()),
        }))
    }

    #[tracing::instrument(skip(self, request), fields(
        remote_addr = request.remote_addr().map(|addr| addr.to_string()),
    ), err(Debug))]
    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, tonic::Status> {
        // let context = Context::from_request(&request);

        let request = ListPostsRequestDto::parse_list_query(
            request.into_inner(),
            self.post_query_manager.list_query_builder(),
        )?;

        let output = self.post_query_manager.query_list(request.query).await?;

        Ok(Response::new(ListPostsResponse {
            posts: output.items.into_iter().collect(),
            next_page_token: output.next_page_token,
            total_size: output.total_size,
        }))
    }
}

impl Debug for PostAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostAdapter").finish()
    }
}
