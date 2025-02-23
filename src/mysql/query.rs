use sqlx::mysql::{MySqlRow, MySqlQueryResult};
use sqlx::{Acquire, Error, FromRow, Pool};

use crate::common::builder::BuilderTrait;
use crate::common::database::DatabaseTrait;
use crate::sql::builder::Builder;
use super::connection;
use super::kind::DataKind;

pub struct MySqlQuery;

impl DatabaseTrait for MySqlQuery {
    type Database = sqlx::MySql;
    type Row = MySqlRow;
    type QueryResult = MySqlQueryResult;
    type QueryBuilder<'a> = Builder<DataKind<'a>>;

    async fn fetch_one<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
    {
        let pool = self.get_db_pool();
        let (sql, values) = qb.build();
        let mut query = sqlx::query_as::<_, T>(&sql);

        // 绑定参数值到查询中
        for value in values {
            query = query.bind(value)
        }

        // 执行查询并返回单条记录
        query.fetch_one(&*pool).await
    }

    async fn fetch_all<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> Result<Vec<T>, Error>
    where
        T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
    {
        let pool = self.get_db_pool();
        let (sql, values) = qb.build();
        dbg!(&sql, &values);
        let mut query = sqlx::query_as::<_, T>(&sql);

        // 绑定参数值到查询中
        for value in values {
            query = query.bind(value)
        }

        // 执行查询并返回多条记录
        query.fetch_all(&*pool).await
    }

    async fn fetch_optional<'a, T>(&self, qb: Self::QueryBuilder<'a>) -> Result<Option<T>, Error>
    where
        T: for<'r> FromRow<'r, MySqlRow> + Unpin + Send,
    {
        let pool = self.get_db_pool();
        let (sql, values) = qb.build();
        let mut query = sqlx::query_as::<_, T>(&sql);

        // 绑定参数值到查询中
        for value in values {
            query = query.bind(value)
        }

        // 执行查询并返回单条可选记录
        query.fetch_optional(&*pool).await
    }

    async fn execute<'a>(&self, qb: Self::QueryBuilder<'a>) -> Result<MySqlQueryResult, Error>{
        let pool = self.get_db_pool();
        let mut conn = pool.acquire().await?;
        let mut tx = conn.begin().await?;
        let (sql, values) = qb.build();
        dbg!(&sql, &values);
        let mut query = sqlx::query(&sql);
        
        // 绑定参数值到查询中
        for value in values {
            query = query.bind(value)
        }
        
        // 执行查询并处理事务
        let result = query.execute(&mut *tx).await;

        match result {
            Ok(r) => {
                // 提交事务
                tx.commit().await?;
                Ok(r)
            },
            Err(e) => {
                // 回滚事务
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    fn get_db_pool(&self) -> &'static Pool<Self::Database> {
        connection::get_db_pool()
    }
}
