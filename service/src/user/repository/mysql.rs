use bomboni_common::id::Id;
use mysql_async::{prelude::*, Pool};

use tracing::info;

use crate::{
    error::AppResult,
    user::repository::{UserInsertRecord, UserRecord, UserRepository},
};

pub struct UserMySqlRepository {
    pool: Pool,
}

impl UserMySqlRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl UserRepository for UserMySqlRepository {
    async fn select(&self, id: Id) -> AppResult<Option<UserRecord>> {
        let mut conn = self.pool.get_conn().await?;

        let users = r#"SELECT id, name, password_hash FROM users WHERE id = :id LIMIT 1"#
            .with(params! {
                "id" => id.to_string(),
            })
            .map(
                &mut conn,
                |(id, name, password_hash): (Id, String, String)| UserRecord {
                    id,
                    name,
                    password_hash,
                },
            )
            .await?;

        Ok(users.into_iter().next())
    }

    async fn select_by_name(&self, name: &str) -> AppResult<Option<UserRecord>> {
        let mut conn = self.pool.get_conn().await?;

        let users = r#"SELECT id, name, password_hash FROM users WHERE name = :name LIMIT 1"#
            .with(params! {
                "name" => name,
            })
            .map(
                &mut conn,
                |(id, name, password_hash): (Id, String, String)| UserRecord {
                    id,
                    name,
                    password_hash,
                },
            )
            .await?;

        Ok(users.into_iter().next())
    }

    async fn insert(&self, record: UserInsertRecord<'_>) -> AppResult<()> {
        let mut conn = self.pool.get_conn().await?;

        r#"INSERT INTO users (id, name, password_hash) VALUES(:id, :name, :password_hash)"#
            .with(params! {
                "id" => record.id,
                "name" => record.name,
                "password_hash" => record.password_hash,
            })
            .ignore(&mut conn)
            .await?;

        info!("inserted user: {:?}", record.id);

        Ok(())
    }
}
