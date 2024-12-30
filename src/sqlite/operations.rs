use std::future::Future;
use std::marker::PhantomData;
use field_access::FieldAccess;
use sqlx::sqlite::SqliteRow;
use sqlx::{Error, FromRow};

use super::kind::{is_empty, value_convert, DataKind};
use super::query;
use super::sql::{field, AggregateFunction, JoinType, SQLBuilder as Builder, WhereClause};

/// 数据操作结构体，用于对数据库中的实体进行增删改查等操作。
pub struct EntityOperations<'a, T>
where 
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send,
{
    /// 实体对象，表示要操作的数据记录。
    entity: T,
    /// 表名，表示实体对应的数据库表。
    table_name: &'a str,
    /// 主键字段名，用于唯一标识表中的记录。
    key_name: &'a str,
    /// 幻影数据，用于编译时类型检查。
    _phantom: PhantomData<&'a T>,
}

/// 定义了对数据库中实体的基本操作接口，包括插入、更新、删除和查询等。
#[allow(dead_code)]
pub trait DataOperations<'a, T> {
    /// 创建一个新的 `EntityOperations` 实例。
    fn new(entity: T, table_name: &'a str, key_name: &'a str) -> Self;

    /// 插入一条新记录到数据库中，并返回插入记录的主键值。
    fn insert(self) -> impl Future<Output = Result<i64, Error>> + Send;

    /// 批量插入多条记录到数据库中，并返回插入记录的主键值列表。
    fn insert_many(self, entities: Vec<T>) -> impl Future<Output = Result<Vec<i64>, Error>> + Send;

    /// 更新数据库中的一条记录，并返回受影响的行数。参数为忽略空值或未提供的字段
    fn update(self, skip_empty_values: bool) -> impl Future<Output = Result<u64, Error>> + Send;

    /// 更新数据库，按case when的方式更新记录，并返回受影响的行数。
    fn update_when<U, V>(
        self,
        column: &str,
        cases: Vec<(WhereClause<DataKind<'a>>, U)>,
        else_value: V,
    ) -> impl Future<Output = Result<u64, Error>> + Send
    where
        U: Into<DataKind<'a>> + Send,
        V: Into<DataKind<'a>> + Send;

    /// 批量更新多条记录，并返回受影响的行数。
    fn update_many<F>(self, entities: Vec<T>) -> impl Future<Output = Result<u64, Error>> + Send;

    /// 删除数据库中的一条记录，并返回受影响的行数。
    fn delete(self) -> impl Future<Output = Result<u64, Error>> + Send;

    /// 删除一定范围内多条记录，并返回受影响的行数。
    fn delete_many(self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> impl Future<Output = Result<u64, Error>> + Send;

    /// 批量删除多条记录，并返回受影响的行数。
    fn batch_delete(self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> impl Future<Output = Result<u64, Error>> + Send;

    /// 查询并返回表中的所有记录。
    fn get_all(self) -> impl Future<Output = Result<Vec<T>, Error>> + Send;

    /// 根据主键查询并返回一条记录。
    fn get_by_key(self) -> impl Future<Output = Result<T, Error>> + Send;

    /// 查询并返回表中的前 N 条记录。
    fn get_top(self, top: i64) -> impl Future<Output = Result<Vec<T>, Error>> + Send;

    /// 分页查询并返回表中的记录。
    fn get_list_paginated(self, page: i64, page_size: i64,) -> impl Future<Output = Result<PaginatedList<T>, Error>> + Send;

    /// 根据查询条件搜索并返回符合条件的记录。
    fn search<F>(self, query_conditions: F) -> impl Future<Output = Result<Vec<T>, Error>> + Send
    where
        F: FnOnce(&mut Builder<'a>) -> Builder<'a> + Send;

    /// 根据查询条件分页搜索并返回符合条件的记录。
    fn search_paginated<F>(
        self,
        query_conditions: F,
        page: i64,
        page_size: i64,
    ) -> impl Future<Output = Result<PaginatedList<T>, Error>> + Send
    where
        F: FnMut(&mut Builder<'a>) -> Builder<'a> + Send;

    /// 检查是否存在满足条件的记录。
    fn exists<F>(self, query_conditions: F) -> impl Future<Output = Result<bool, Error>> + Send
    where
        F: FnOnce(&mut Builder<'a>) -> Builder<'a> + Send;

    /// 检查某个字段的值是否唯一。
    fn is_unique(self, column: &str, value: impl Into<DataKind<'a>> + Send) -> impl Future<Output = Result<bool, Error>> + Send;

    /// 执行自定义的 SQL 语句。
    fn execute(self, custom_builder: Builder<'a>) -> impl Future<Output = Result<u64, Error>> + Send;

    /// 执行聚合函数查询，支持 COUNT, SUM, AVG 等。
    fn agg<F>(&self, agg_function: AggregateFunction, column: &str, query_conditions: F) -> impl Future<Output = Result<f64, Error>> + Send
    where
        F: FnOnce(&mut Builder<'a>) -> Builder<'a> + Send;

    /// 执行 JOIN 查询，支持 INNER JOIN, LEFT JOIN, RIGHT JOIN, FULL OUTER JOIN。
    fn join<F>(
        self,
        join_type: JoinType,
        table: &str,
        condition: &str,
        query_conditions: F,
    ) -> impl Future<Output = Result<Vec<T>, Error>> + Send
    where
        F: FnOnce(&mut Builder<'a>) -> Builder<'a> + Send;
}

/// 实现了 `DataOperations` 接口，提供了具体的数据库操作方法。
impl<'a, T> DataOperations<'a, T> for EntityOperations<'a, T> 
where 
    T: for<'r> FromRow<'r, SqliteRow> + FieldAccess + Unpin + Send + Sync,
{
    /// 创建一个新的 `EntityOperations` 实例。
    fn new(entity: T, table_name: &'a str, key_name: &'a str) -> Self {
        EntityOperations {
            entity,
            table_name,
            key_name,
            _phantom: PhantomData,
        }
    }

    /// 插入一条新记录到数据库中，并返回插入记录的主键值。
    async fn insert(self) -> Result<i64, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();

        // 收集需要插入的字段名和字段值
        for (name, field) in self.entity.fields() {            
            if name != self.key_name {
                cols_names.push(name);
                let value = value_convert(field.as_any());
                cols_values.push(value);
            }
        }
        
        // 构建并执行插入语句
        let sb = Builder::insert_into(&self.table_name, &cols_names, vec![cols_values]);
        let result = query::execute(sb).await?;
        Ok(result.last_insert_rowid())
    }

    /// 批量插入多条记录到数据库中，并返回插入记录的主键值列表。
    async fn insert_many(self, entities: Vec<T>) -> Result<Vec<i64>, Error> {
        if entities.is_empty() {
            return Ok(vec![]);
        }
        let mut values_list = Vec::new();

        // 收集需要插入的字段名
        let mut cols_names = self.entity.field_names().to_vec();
        if let Some(index) = cols_names.iter().position(|&name| name == self.key_name) {
            cols_names.remove(index);
        }

        // 收集所有记录的字段值
        for entity in entities.iter() {
            let mut row_values = Vec::new();
            for (name, field) in entity.fields() {
                if name != self.key_name {
                    let value = value_convert(field.as_any());
                    row_values.push(value);
                }
            }
            values_list.push(row_values);
        }

        // 构建并执行批量插入语句
        let sb = Builder::insert_into(&self.table_name, &cols_names, values_list);
        let result = query::execute(sb).await?;

        Ok((0..result.rows_affected()).map(|_| result.last_insert_rowid()).collect())
    }

    /// 更新数据库中的一条记录，并返回受影响的行数。参数 `skip_empty_values`，忽略空值或未提供的字段
    async fn update(self, skip_empty_values: bool) -> Result<u64, Error> {
        let mut cols_names = Vec::new();
        let mut cols_values = Vec::new();
        let mut last = None;

        // 收集需要更新的字段名和字段值
        for (name, field) in self.entity.fields() {
            let value = field.as_any();            
            if name != self.key_name {                
                if !skip_empty_values || !is_empty(value) {
                    cols_names.push(name);
                    let value = value_convert(value);
                    cols_values.push(value);
                }
            } else {
                last = Some(value_convert(value));
            }
        }

        // 构建并执行更新语句
        if let Some(v) = last {
            let sb = Builder::update(&self.table_name, &cols_names, cols_values)
                .filter(field(self.key_name).eq(v));
            let result = query::execute(sb).await?;
            Ok(result.rows_affected())
        } else {
            Err(Error::RowNotFound)
        }
    }

    /// 按case_when 语句查询更新数据。
    async fn update_when<U, V>(
        self,
        column: &str,
        cases: Vec<(WhereClause<DataKind<'a>>, U)>,
        else_value: V,
    ) -> Result<u64, Error>
    where
        U: Into<DataKind<'a>> + Send,
        V: Into<DataKind<'a>> + Send,
    {
        // 获取主键值
        let mut key_value = None;
        for (name, field) in self.entity.fields() {
            if name == self.key_name {
                key_value = Some(value_convert(field.as_any()));
                break;
            }
        }

        // 如果没有主键值，返回错误
        let key_value = key_value.ok_or(Error::RowNotFound)?;

        // 将 cases 中的值转换为 DataKind
        let cases: Vec<(WhereClause<DataKind<'a>>, DataKind<'a>)> = cases
            .into_iter()
            .map(|(condition, value)| (condition, value.into()))
            .collect();

        // 将 else_value 转换为 DataKind
        let else_value = else_value.into();

        // 构建更新语句
        let sb = Builder::update(self.table_name, &[], vec![])
            .case_when(column, cases, else_value)
            .filter(field(self.key_name).eq(key_value));

        // 执行更新操作
        let result = query::execute(sb).await?;
        Ok(result.rows_affected())
    }


    /// 批量更新多条记录，并返回受影响的行数。
    async fn update_many<F>(self, entities: Vec<T>) -> Result<u64, Error> 
    {
        if entities.is_empty() {
            return Ok(0);
        }

        let mut total_rows_affected = 0;

        for entity in entities {
            let entity_ops = EntityOperations::new(entity, self.table_name, self.key_name);
            let mut cols_names = Vec::new();
            let mut cols_values = Vec::new();
            let mut last = None;

            // 收集需要更新的字段名和字段值
            for (name, field) in entity_ops.entity.fields() {
                let value = field.as_any();
                if name != entity_ops.key_name {
                    cols_names.push(name);
                    let value = value_convert(value);
                    cols_values.push(value);
                } else {
                    last = Some(value_convert(value));
                }
            }

            // 构建并执行更新语句
            let mut sb = Builder::update(&entity_ops.table_name, &cols_names, cols_values);
            if let Some(v) = last {
                sb = sb.filter(field(entity_ops.key_name).eq(v));
            }
            let result = query::execute(sb).await?;
            total_rows_affected += result.rows_affected();
        }

        Ok(total_rows_affected)
    }
    
    /// 删除数据库中的一条记录，并返回受影响的行数。
    async fn delete(self) -> Result<u64, Error> {
        let mut last = None;

        // 获取主键值
        for (name, field) in self.entity.fields() {
            if name == self.key_name {
                last = Some(value_convert(field.as_any()));
                break;
            }
        }
    
        // 构建并执行删除语句
        let mut sb = Builder::delete(&self.table_name);
        if let Some(v) = last {
            sb = sb.filter(field(self.key_name).eq(v));
        }
        let result = query::execute(sb).await?;
        Ok(result.rows_affected())
    }

    /// 删除一定范围内的多条记录，并返回受影响的行数。
    async fn delete_many(self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<u64, Error> {        
        let converted_keys: Vec<DataKind<'a>> = keys.into_iter().map(|k| k.into()).collect();
        let mut sb = Builder::delete(&self.table_name);
        sb = sb.filter(field(self.key_name).in_list(converted_keys));
        let result = query::execute(sb).await?;
        Ok(result.rows_affected())
    }

    /// 批量删除多条记录，并返回受影响的行数。
    async fn batch_delete(self, keys: Vec<impl Into<DataKind<'a>> + Send>) -> Result<u64, Error> {
        // 定义每批删除的大小
        const BATCH_SIZE: usize = 100;
        let mut total_rows_affected = 0;
    
        // 分批处理删除操作
        for chunk in keys.into_iter().map(|k| k.into()).collect::<Vec<DataKind<'a>>>().chunks(BATCH_SIZE) {
            let mut sb = Builder::delete(&self.table_name);
            sb = sb.filter(field(self.key_name).in_list(chunk.to_vec()));
            let result = query::execute(sb).await?;
            total_rows_affected += result.rows_affected();
        }
    
        Ok(total_rows_affected)
    }

    /// 查询并返回表中的所有记录，倒序。
    async fn get_all(self) -> Result<Vec<T>, Error> {
        let names = self.entity.field_names();
        let sb = Builder::select(&self.table_name, names)
            .order_by(self.key_name, false);
    
        let rows = query::fetch_all::<T>(sb).await?;
        Ok(rows)
    }

    /// 根据主键查询并返回一条记录。
    async fn get_by_key(self) -> Result<T, Error> {    
        let mut cols_names = Vec::new();
        let mut last = None;

        // 获取主键值
        for (name, field) in self.entity.fields() {            
            if name == self.key_name {
            last = Some(value_convert(field.as_any()));
            }
            cols_names.push(name);
        }

        // 构建并执行查询语句
        let mut sb = Builder::select(&self.table_name, &cols_names);
        if let Some(v) = last {
            sb = sb.filter(field(self.key_name).eq(v));
        }

        let result  = query::fetch_optional::<T>(sb).await?;
        match result {
            Some(ret) => Ok(ret),
            None => Err(Error::RowNotFound),
        }
    }

    /// 查询并返回表中的前 N 条记录。
    async fn get_top(self, top: i64) -> Result<Vec<T>, Error> {
        let names = self.entity.field_names();
        let sb = Builder::select(&self.table_name, names)
            .order_by(self.key_name, false)
            .limit(top);
    
        let rows = query::fetch_all::<T>(sb).await?;
        Ok(rows)
    }

    /// 分页查询并返回表中的记录，倒序。
    async fn get_list_paginated(self, page: i64, page_size: i64,) -> Result<PaginatedList<T>, Error> {
        let names = self.entity.field_names();
        let sb = Builder::select(&self.table_name, names)
            .order_by(self.key_name, false)
            .paginate(page, page_size);
    
        let sb1 = Builder::count(&self.table_name);
        let items: Vec<T> = query::fetch_all(sb).await?;    
        let result: (i64,) = query::fetch_one(sb1).await?;
        let total = result.0;

        Ok(PaginatedList {
            items,
            total,
            page,
            page_size,
        })
    }    

    /// 根据查询条件搜索并返回符合条件的记录。
    async fn search<F>(self, query_conditions: F) -> Result<Vec<T>, Error>
    where
        F: for<'b> FnOnce(&'b mut Builder<'a>) -> Builder<'a> + Send,
    {
        let names = self.entity.field_names();
        let mut sb = Builder::select(&self.table_name, names);
        sb = (query_conditions)(&mut sb);

        let rows = query::fetch_all::<T>(sb).await?;
        Ok(rows)
    }

    /// 根据查询条件分页搜索并返回符合条件的记录。
    async fn search_paginated<F>(
        self,        
        mut query_conditions: F,
        page: i64,
        page_size: i64,
    ) -> Result<PaginatedList<T>, Error>
    where
        F: for<'b> FnMut(&'b mut Builder<'a>) -> Builder<'a> + Send,
    {
        let names = self.entity.field_names();
        let mut sb = Builder::select(&self.table_name, names)
            .paginate(page, page_size);
        sb = (query_conditions)(&mut sb);

        let mut sb1 = Builder::count(&self.table_name);
        sb1 = (query_conditions)(&mut sb1);

        let items: Vec<T> = query::fetch_all(sb).await?;    
        let result: (i64,) = query::fetch_one(sb1).await?;
        let total = result.0;
    
        Ok(PaginatedList {
            items,
            total,
            page,
            page_size,
        })
    }

    /// 检查是否存在满足条件的记录。
    async fn exists<F>(self, query_conditions: F) -> Result<bool, Error>
    where
        F: FnOnce(&mut Builder<'a>) -> Builder<'a> + Send,
    {
        let mut sb = Builder::select(&self.table_name, &["1"])
            .limit(1);
        sb = (query_conditions)(&mut sb);

        let result = query::fetch_optional::<(i32,)>(sb).await?;
        Ok(result.is_some())
    }

    /// 检查某个字段的值是否唯一。
    async fn is_unique(self, column: &str, value: impl Into<DataKind<'a>> + Send) -> Result<bool, Error> {
        let value = value.into();
        let sb = Builder::select(&self.table_name, &["1"])
            .filter(field(column).eq(value))
            .limit(1);

        let result = query::fetch_optional::<(i32,)>(sb).await?;
        Ok(result.is_none())
    }

    /// 执行自定义的 SQL 语句。
    async fn execute(self, custom_builder: Builder<'a>) -> Result<u64, Error> {
        let result = query::execute(custom_builder).await?;
        Ok(result.rows_affected())
    }

    /// 执行聚合函数查询，支持 COUNT, SUM, AVG 等。
    async fn agg<F>(&self, agg_function: AggregateFunction, column: &str, query_conditions: F) -> Result<f64, Error>
    where
        F: FnOnce(&mut Builder<'a>) -> Builder<'a> + Send,
    {
        let mut sb = Builder::agg(self.table_name, agg_function, column);

        // 应用查询条件
        sb = query_conditions(&mut sb);

        // 执行查询
        let result: Option<(f64,)> = query::fetch_optional(sb).await?;
        match result {
            Some((value,)) => Ok(value),
            None => Ok(0.0), // 如果没有结果，返回 0.0
        }
    }

    /// 执行 JOIN 查询，支持 INNER JOIN, LEFT JOIN, RIGHT JOIN, FULL OUTER JOIN。
    async fn join<F>(
        self,
        join_type: JoinType,
        table: &str,
        condition: &str,
        query_conditions: F,
    ) -> Result<Vec<T>, Error>
    where
        F: FnOnce(&mut Builder<'a>) -> Builder<'a> + Send,
    {
        let mut sb = Builder::select(self.table_name, &self.entity.field_names())
            .join(join_type, table, condition);

        // 应用查询条件
        sb = query_conditions(&mut sb);

        // 执行查询
        let rows = query::fetch_all::<T>(sb).await?;
        Ok(rows)
    }
    

}

/// 分页信息结构体，用于存储分页查询的结果。
#[derive(Debug)]
pub struct PaginatedList<T> {
    /// 查询结果的记录列表。
    pub items: Vec<T>,
    /// 总记录数。
    pub total: i64,
    /// 当前页码。
    pub page: i64,
    /// 每页显示的记录数。
    pub page_size: i64,
}