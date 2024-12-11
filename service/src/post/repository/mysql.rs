use bomboni_common::btree_map_into;
use bomboni_common::{date_time::UtcDateTime, id::Id};
use bomboni_request::{
    error::RequestError,
    query::{error::QueryError, list::ListQuery},
    sql::{QuerySqlBuilder, SqlArgumentStyle, SqlDialect, SqlRenameMap},
};
use mysql_async::{prelude::*, Pool};
use std::collections::BTreeMap;
use tracing::info;

use grpc_sky_api::dto::PostDto;

use crate::{
    error::AppResult,
    post::repository::{PostInsertRecord, PostRecord, PostRecordList, PostRepository},
};

pub struct PostMySqlRepository {
    pool: Pool,
    query_sql_builder: QuerySqlBuilder,
}

impl PostMySqlRepository {
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            query_sql_builder: {
                let mut query_sql_builder =
                    QuerySqlBuilder::new(SqlDialect::MySql, PostDto::get_schema());
                query_sql_builder
                    .set_argument_style(SqlArgumentStyle::Positional { symbol: "?".into() });
                query_sql_builder.set_rename_map(SqlRenameMap {
                    members: btree_map_into! {
                        "userId" => "user_id",
                        "createTime" => "create_time",
                    },
                    functions: BTreeMap::new(),
                });
                query_sql_builder
            },
        }
    }
}

#[tonic::async_trait]
impl PostRepository for PostMySqlRepository {
    async fn select(&self, id: Id) -> AppResult<Option<PostRecord>> {
        let mut conn = self.pool.get_conn().await?;

        let users = r#"SELECT id, user_id, content, create_time FROM posts WHERE id = :id LIMIT 1"#
            .with(params! {
                "id" => id,
            })
            .map(
                &mut conn,
                |(id, user_id, content, create_time): (Id, Id, String, UtcDateTime)| PostRecord {
                    id,
                    user_id,
                    content,
                    create_time,
                },
            )
            .await?;

        Ok(users.into_iter().next())
    }

    async fn select_list(&self, query: &ListQuery) -> AppResult<PostRecordList> {
        let query_statement = self
            .query_sql_builder
            .build_list(&query)
            .map_err(|err: QueryError| RequestError::generic(err))?;

        let statement = format!(
            r#"
                SELECT
                    id,
                    user_id,
                    content,
                    create_time
                FROM posts
                {}
                ORDER BY {}
                {}
            "#,
            if let Some(paged_where_clause) = query_statement.paged_where_clause.as_ref() {
                format!("WHERE ({})", paged_where_clause)
            } else {
                String::new()
            },
            if let Some(order_by_clause) = query_statement.order_by_clause.as_ref() {
                order_by_clause
            } else {
                "id"
            },
            &query_statement.paged_limit_clause,
        );

        let total_count_statement = format!(
            r#"
                SELECT COUNT(id)
                FROM posts
                {}
            "#,
            if let Some(where_clause) = query_statement.where_clause.as_ref() {
                format!("WHERE ({})", where_clause)
            } else {
                String::new()
            },
        );

        let mut conn = self.pool.get_conn().await?;

        let mut items = {
            let params: Vec<mysql_async::Value> = query_statement
                .paged_arguments
                .into_iter()
                .map(Into::into)
                .collect();

            statement
                .with(params)
                .map(
                    &mut conn,
                    |(id, user_id, content, create_time): (Id, Id, String, UtcDateTime)| {
                        PostRecord {
                            id,
                            user_id,
                            content,
                            create_time,
                        }
                    },
                )
                .await?
        };

        let total_count_result = {
            let params: Vec<mysql_async::Value> = query_statement
                .arguments
                .into_iter()
                .map(Into::into)
                .collect();

            total_count_statement
                .with(params)
                .map(&mut conn, |(total_count,): (i64,)| total_count)
                .await?
        };

        let total_size = total_count_result.first().copied().unwrap_or_default();
        let next_item = if items.len() > query.page_size as usize {
            items.pop()
        } else {
            None
        };

        Ok(PostRecordList {
            items,
            next_item,
            total_size,
        })
    }

    async fn insert(&self, record: PostInsertRecord<'_>) -> AppResult<()> {
        let mut conn = self.pool.get_conn().await?;

        r#"INSERT INTO posts (id, user_id, content, create_time) VALUES(:id, :user_id, :content, :create_time)"#
            .with(params! {
                "id" => record.id,
                "user_id" => record.user_id,
                "content" => record.content,
                "create_time" => record.create_time,
            })
            .ignore(&mut conn)
            .await?;

        info!("inserted user: {:?}", record.id);

        Ok(())
    }
}
