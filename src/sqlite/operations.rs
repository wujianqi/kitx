use std::marker::PhantomData;
use field_access::FieldAccess;
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{Error, FromRow, Sqlite};

use crate::common::builder::BuilderTrait;
use crate::common::database::DatabaseTrait;
use crate::common::operations::{OperationsTrait, CursorPaginatedResult, PaginatedResult};

use super::kind::{is_empty, value_convert, DataKind};
use super::query::SqliteQuery;
use super::sql::{field, QueryBuilder, QueryCondition};

/// 数据操作结构体，用于对数据库中的实体进行增删改查等操作。
pub struct Operations<'a, T>
where
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send,
{
    /// 表名，表示实体对应的数据库表。
    table_name: &'a str,
    /// 主键字段名，用于唯一标识表中的记录。
    primary_key: &'a str,
    /// 软删除字段名和过滤标志，用于标记记录是否已删除及是否过滤。
    soft_delete_info: Option<(&'a str, bool)>,
    /// 幻影数据，用于编译时类型检查。
    _phantom: PhantomData<&'a T>,

    query: SqliteQuery,
}

impl<'a, T> OperationsTrait<'a, T, Sqlite> for Operations<'a, T>
where
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send + Sync + Default,
{
        
    type Query = QueryCondition<'a>;    
    type DataKind = DataKind<'a>;
    type QueryResult = SqliteQueryResult;
    fn new(table_name: &'a str, primary_key: &'a str, soft_delete_info: Option<(&'a str, bool)>) -> Self {
        Operations {
            table_name,
            primary_key,
            soft_delete_info,
            _phantom: PhantomData,
            query: SqliteQuery,
        }
    }

    async fn insert_one(&self, entity: T) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        for (name, field) in entity.fields() {
            if name != self.primary_key {
                cols_names.push(name);
                let value = value_convert(field.as_any());
                cols_values.push(value);
            }
        }

        let query = QueryBuilder::insert_into(self.table_name, &cols_names, vec![cols_values]);
        self.query.execute(query).await
    }

    async fn insert_many(&self, entities: Vec<T>) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut all_cols_values = Vec::new();

        for entity in entities {
            let mut cols_values = Vec::new();
            for (name, field) in entity.fields() {
                if name != self.primary_key {
                    if cols_names.is_empty() {
                        cols_names.push(name);
                    }
                    let value = value_convert(field.as_any());
                    cols_values.push(value);
                }
            }
            all_cols_values.push(cols_values);
        }

        let query = QueryBuilder::insert_into(self.table_name, &cols_names, all_cols_values);
        self.query.execute(query).await
    }

    async fn update_one(&self, entity: T, override_empty: bool) -> Result<Self::QueryResult, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();
    
        // 第一部分：收集更新字段
        for (name, field) in entity.fields() {
            if name != self.primary_key {
                let value = value_convert(field.as_any());
                if !override_empty && is_empty(&value) {
                    continue;
                }
                cols_names.push(name);
                cols_values.push(value);
            }
        }
    
        // 第二部分：优化后的主键获取
        let primary_key_value = entity.fields()
            .find(|(name, _)| *name == self.primary_key)
            .map(|(_, field)| value_convert(field.as_any()))
            .ok_or(Error::RowNotFound)?;
    
        // 第三部分：构建查询
        let mut query = QueryBuilder::update(self.table_name, &cols_names, cols_values);
        query.filter(field(self.primary_key).eq(primary_key_value.clone()));
    
        self.query.execute(query).await
    }

    async fn update_many(&self, entities: Vec<T>, override_empty: bool) -> Result<Vec<Self::QueryResult>, Error> {
        let mut results = Vec::new();
        for entity in entities {
            let mut cols_names = Vec::new();
            let mut cols_values = Vec::new();

            for (name, field) in entity.fields() {
                if name != self.primary_key {
                    let value = value_convert(field.as_any());
                    if override_empty && is_empty(&value) {
                        continue;
                    }
                    cols_names.push(name);
                    cols_values.push(value);
                }
            }

            let primary_key_value = {
                let mut primary_key_value = None;
                for (name, field) in entity.fields() {
                    if name == self.primary_key {
                        primary_key_value = Some(value_convert(field.as_any()));
                        break;
                    }
                }
                primary_key_value.ok_or(Error::RowNotFound)?
            };

            let mut query = QueryBuilder::update(self.table_name, &cols_names, cols_values);
            query.filter(field(self.primary_key).eq(primary_key_value));
            let result = self.query.execute(query).await?;
            results.push(result);
        }
        Ok(results)
    }

    async fn delete_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();

        if let Some((column, _)) = self.soft_delete_info {
            let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(true)]);
            query.filter(field(self.primary_key).eq(key));
            self.query.execute(query).await
        } else {
            let mut query = QueryBuilder::delete(self.table_name);
            query.filter(field(self.primary_key).eq(key));
            self.query.execute(query).await
        }
    }

    async fn delete_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        if let Some((column, _)) = self.soft_delete_info {
            let mut query = QueryBuilder::update(self.table_name, &[column], vec![DataKind::from(true)]);
            query.filter(field(self.primary_key).in_list(keys));
            self.query.execute(query).await
        } else {
            let mut query = QueryBuilder::delete(self.table_name);
            query.filter(field(self.primary_key).in_list(keys));
            self.query.execute(query).await
        }
    }

    async fn restore_one(&self, key: impl Into<DataKind<'a>> + Send) -> Result<Self::QueryResult, Error> {
        let key = key.into();
        let mut query = QueryBuilder::update(self.table_name, &[self.soft_delete_info.as_ref().unwrap().0], vec![DataKind::from(false)]);
        query.filter(field(self.primary_key).eq(key));
        self.query.execute(query).await
    }

    async fn restore_many(&self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<Self::QueryResult, Error> {
        let keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        let mut query = QueryBuilder::update(self.table_name, &[self.soft_delete_info.as_ref().unwrap().0], vec![DataKind::from(false)]);
        query.filter(field(self.primary_key).in_list(keys));
        self.query.execute(query).await
    }

    async fn fetch_all(&self, query_condition: Self::Query) -> Result<Vec<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        self.apply_soft_delete_filter(&mut builder);
        let result = self.query.fetch_all::<T>(builder).await?;
        Ok(result)
    }

    async fn fetch_by_key(&self, id: impl Into<DataKind<'a>> + Send) -> Result<Option<T>, Error> {
        let id = id.into();
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        builder.filter(field(self.primary_key).eq(id));
        self.apply_soft_delete_filter(&mut builder);
        self.query.fetch_optional::<T>(builder).await
    }

    async fn fetch_one(&self, query_condition: Self::Query) -> Result<Option<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        self.apply_soft_delete_filter(&mut builder);
        self.query.fetch_optional::<T>(builder).await
    }

    async fn fetch_paginated(&self, page_number: u64, page_size: u64, query_condition: Self::Query) -> Result<PaginatedResult<T>, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);       
        self.apply_soft_delete_filter(&mut builder);

        let offset = (&page_number - 1) * &page_size;
        builder.limit_offset(page_size, Some(offset));

        let data = self.query.fetch_all::<T>(builder).await?;
        let total = self.count(query_condition).await?;
        Ok(PaginatedResult { data, total, page_number, page_size })
    }

    async fn fetch_by_cursor(&self, limit: u64, query_condition: Self::Query) -> Result<CursorPaginatedResult<T>, Error>
    where
        T: Clone,
    {
        let mut builder = QueryBuilder::select(self.table_name, &["*"]);
        query_condition.apply(&mut builder);
        self.apply_soft_delete_filter(&mut builder);

        builder.limit_offset(limit,None);
        let data = self.query.fetch_all::<T>(builder).await?;

        // 获取最后一个记录的游标值
        let next_cursor = data.last().cloned();

        Ok(CursorPaginatedResult {
            data,
            next_cursor,
            page_size: limit,
        })
    }

    async fn exist(&self, query_condition: Self::Query) -> Result<bool, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["1"]);
        query_condition.apply(&mut builder);
        self.apply_soft_delete_filter(&mut builder);
        let result = self.query.fetch_optional::<(i32,)>(builder).await?;
        Ok(result.is_some())
    }

    async fn count(&self, query_condition: Self::Query) -> Result<i64, Error> {
        let mut builder = QueryBuilder::select(self.table_name, &["COUNT(*)"]);
        query_condition.apply(&mut builder);
        self.apply_soft_delete_filter(&mut builder);
        let result = self.query.fetch_one::<(i64,)>(builder).await?;
        Ok(result.0)
    }

}

impl<'a, T> Operations<'a, T>
where
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send + Sync + Default,
{
    // 应用软删除内容过滤
    fn apply_soft_delete_filter(&self, builder: &mut QueryBuilder<'a>) {
        if let Some((soft_delete_field, filter_soft_deleted)) = &self.soft_delete_info {
            if *filter_soft_deleted {
                builder.filter(field(soft_delete_field).eq(false));
            }
        }
    }
}
