use bomboni_common::id::Id;
use std::sync::Arc;

use crate::error::AppResult;

pub mod mysql;

pub struct UserRecord {
    pub id: Id,
    pub name: String,
    pub password_hash: String,
}

pub struct UserInsertRecord<'a> {
    pub id: Id,
    pub name: &'a str,
    pub password_hash: String,
}

#[tonic::async_trait]
pub trait UserRepository {
    async fn select(&self, id: Id) -> AppResult<Option<UserRecord>>;

    async fn select_by_name(&self, name: &str) -> AppResult<Option<UserRecord>>;

    async fn insert(&self, record: UserInsertRecord<'_>) -> AppResult<()>;
}

pub type UserRepositoryArc = Arc<dyn UserRepository + Send + Sync>;
