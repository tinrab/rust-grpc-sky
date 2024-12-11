use bomboni_request::{
    ordering::{OrderingDirection, OrderingTerm},
    query::{
        list::{Aes256ListQueryBuilder, ListQuery, ListQueryConfig},
        page_token::aes256::Aes256PageTokenBuilder,
    },
    schema::FunctionSchemaMap,
};
use grpc_sky_api::{dto::PostDto, proto::Post};

use crate::{error::AppResult, post::repository::PostRepositoryArc};

pub struct PostQueryManager {
    post_repository: PostRepositoryArc,
    list_query_builder: Aes256ListQueryBuilder,
}

pub struct PostListResult {
    pub items: Vec<Post>,
    pub next_page_token: Option<String>,
    pub total_size: i64,
}

impl PostQueryManager {
    pub fn new(post_repository: PostRepositoryArc) -> Self {
        Self {
            post_repository,
            list_query_builder: Aes256ListQueryBuilder::new(
                PostDto::get_schema(),
                FunctionSchemaMap::new(),
                ListQueryConfig {
                    max_page_size: Some(20),
                    default_page_size: 5,
                    primary_ordering_term: Some(OrderingTerm {
                        name: "id".into(),
                        direction: OrderingDirection::Descending,
                    }),
                    max_filter_length: Some(100),
                    max_ordering_length: Some(100),
                },
                Aes256PageTokenBuilder::new(true),
            ),
        }
    }

    pub async fn query_list(&self, query: ListQuery) -> AppResult<PostListResult> {
        // query.filter.add_conjunction(
        //     Filter::Restriction(
        //         Box::new(Filter::Name("userId".into())),
        //         FilterComparator::Equal,
        //         Box::new(Filter::Value(user_id.to_string().into())),
        //     )
        //     .into(),
        // );
        let post_list = self.post_repository.select_list(&query).await?;

        let next_page_token = if let Some(next_item) = &post_list.next_item {
            Some(
                self.list_query_builder
                    .build_next_page_token(&query, next_item)
                    .unwrap(),
            )
        } else {
            None
        };

        Ok(PostListResult {
            items: post_list
                .items
                .into_iter()
                // TODO: apply views
                .map(|record| Post {
                    id: record.id.to_string(),
                    user_id: record.user_id.to_string(),
                    content: record.content,
                    create_time: Some(record.create_time.into()),
                })
                .collect(),
            next_page_token,
            total_size: post_list.total_size,
        })
    }

    pub fn list_query_builder(&self) -> &Aes256ListQueryBuilder {
        &self.list_query_builder
    }
}
