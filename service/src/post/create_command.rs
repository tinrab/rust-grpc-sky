use bomboni_common::{date_time::UtcDateTime, id::Id};

use crate::{
    context::Context,
    error::AppResult,
    post::repository::{PostInsertRecord, PostRepositoryArc},
};

pub struct CreatePostCommand {
    post_repository: PostRepositoryArc,
}

pub struct CreatePostCommandInput<'a> {
    pub content: &'a str,
}

pub struct CreatePostCommandOutput {
    pub post_id: Id,
    pub create_time: UtcDateTime,
}

impl CreatePostCommand {
    pub fn new(post_repository: PostRepositoryArc) -> Self {
        Self { post_repository }
    }

    pub async fn execute(
        &self,
        context: &Context,
        input: CreatePostCommandInput<'_>,
    ) -> AppResult<CreatePostCommandOutput> {
        let user_id = context.authenticate()?;

        let post_id = Id::generate();
        let create_time = UtcDateTime::now();

        self.post_repository
            .insert(PostInsertRecord {
                id: post_id,
                user_id,
                content: input.content.into(),
                create_time,
            })
            .await?;

        Ok(CreatePostCommandOutput {
            post_id,
            create_time,
        })
    }
}
