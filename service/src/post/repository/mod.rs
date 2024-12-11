use bomboni_common::{date_time::UtcDateTime, id::Id};
use bomboni_request::{query::list::ListQuery, schema::SchemaMapped, value::Value as FilterValue};
use std::sync::Arc;

use crate::error::AppResult;

pub mod mysql;

#[derive(Debug, Clone)]
pub struct PostRecord {
    pub id: Id,
    pub user_id: Id,
    pub content: String,
    pub create_time: UtcDateTime,
}

pub struct PostInsertRecord<'a> {
    pub id: Id,
    pub user_id: Id,
    pub content: &'a str,
    pub create_time: UtcDateTime,
}

#[derive(Debug, Clone)]
pub struct PostRecordList {
    pub items: Vec<PostRecord>,
    pub next_item: Option<PostRecord>,
    pub total_size: i64,
}

#[tonic::async_trait]
pub trait PostRepository {
    async fn select(&self, id: Id) -> AppResult<Option<PostRecord>>;

    async fn select_list(&self, query: &ListQuery) -> AppResult<PostRecordList>;

    async fn insert(&self, record: PostInsertRecord<'_>) -> AppResult<()>;
}

pub type PostRepositoryArc = Arc<dyn PostRepository + Send + Sync>;

impl SchemaMapped for PostRecord {
    fn get_field(&self, name: &str) -> FilterValue {
        match name {
            "id" => self.id.to_string().into(),
            "userId" => self.user_id.to_string().into(),
            _ => unimplemented!("SchemaMapped for PostRecord::{}", name),
        }
    }
}
