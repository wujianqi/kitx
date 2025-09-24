use std::{iter::once, marker::PhantomData};

use field_access::FieldAccess;
use sqlx::{Database, Encode, Error, QueryBuilder, Type};

use crate::common::{
    conversion::ValueConvert, error::QueryError, fields::batch_extract, helper::get_table_name, types::PrimaryKey
};

/// INSERT 查询构建器
/// 
/// 提供直观的 API 来构建 INSERT SQL 查询。
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess trait 的实体类型
/// * `DB` - 实现 sqlx::Database trait 的数据库类型
/// * `VAL` - 实现 Encode 和 Type traits 的值类型
pub struct Insert<'a, ET, DB, VAL>
where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + 'a,
{
    query_builder: QueryBuilder<'a, DB>,
    columns_specified: bool,
    _phantom: PhantomData<(ET, VAL)>,
}

impl<'a, ET, DB, VAL> Insert<'a, ET, DB, VAL>
where
    ET: FieldAccess,
    DB: Database,
    VAL: Encode<'a, DB> + Type<DB> + ValueConvert + 'a,
{
    /// 开始构建 INSERT 查询（使用实体的默认表名）
    /// 
    /// # 返回值
    /// 新的 Insert 构建器实例
    /// 
    /// # 示例
    /// ```
    /// let insert = Insert::<User, Postgres>::table();
    /// ```
    pub fn table() -> Self {
        let table_name = get_table_name::<ET>();
        Self::with_table(&table_name)
    }

    /// 开始构建 INSERT 查询（指定表名）
    /// 
    /// # 参数
    /// * `table_name` - 要插入的表名
    /// 
    /// # 返回值
    /// 新的 Insert 构建器实例
    pub fn with_table(table_name: impl Into<String>) -> Self {
        Self::from_query_with_table(QueryBuilder::new(""), table_name)
    }

    /// 从外部查询构建器创建 INSERT 构建器（使用默认表名）
    pub fn from_query(qb: QueryBuilder<'a, DB>) -> Self {
        Self::from_query_with_table(qb, &get_table_name::<ET>())
    }

    /// 从外部查询构建器创建 INSERT 构建器（指定表名）
    pub fn from_query_with_table(mut query_builder: QueryBuilder<'a, DB>, table_name: impl Into<String>) -> Self {
        query_builder.push("INSERT INTO ").push(table_name.into());

        Self {
            query_builder,
            columns_specified: false,
            _phantom: PhantomData,
        }
    }

    /// 指定要插入的列
    /// 
    /// # 参数
    /// * `columns` - 列名集合
    /// 
    /// # 返回值
    /// 更新后的构建器实例
    pub fn columns<I, S>(mut self, columns: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let cols: Vec<String> = columns.into_iter().map(|s| s.as_ref().to_string()).collect();
        if !cols.is_empty() {
            self.query_builder.push(" (");
            let mut separated = self.query_builder.separated(", ");
            for col in cols {
                separated.push(col);
            }
            self.query_builder.push(")");
            self.columns_specified = true;
        }
        self
    }

    /// Create multiple records insert operation
    /// 
    /// # Arguments
    /// * `models` - Collection of entity models to insert
    /// * `primary_key` - Primary key definition
    /// 
    /// # Returns
    /// A QueryBuilder with the INSERT query or an Error
    /// 
    /// 创建多条记录插入操作
    /// 
    /// # 参数
    /// * `models` - 要插入的实体模型集合
    /// * `primary_key` - 主键定义
    /// 
    /// # 返回值
    /// 包含 INSERT 查询的 QueryBuilder 或错误
    pub fn many(
        models: impl IntoIterator<Item = &'a ET>, 
        primary_key: &PrimaryKey<'a>
    ) -> Result<QueryBuilder<'a, DB>, Error>
    {
        let models: Vec<_> = models.into_iter().collect();
        if models.is_empty() {
            return Err(QueryError::NoEntitiesProvided.into());
        }

        let keys = if primary_key.auto_generate() {
            primary_key.get_keys()
        } else {
            vec![]
        };
        let (names, values) = batch_extract::<ET, VAL>(&models, &keys, false);
        let mut query_builder = Self::table().query_builder;
        query_builder.push(" (").push(names.join(", ")).push(") ");
        query_builder.push_values(
            values,
            |mut b, row| {
                for value in row {
                    b.push_bind(value);
                }
            }
        );

        Ok(query_builder)
    }

    /// Create single record insert operation
    /// 
    /// # Arguments
    /// * `model` - Entity model to insert
    /// * `primary_key` - Primary key definition
    /// 
    /// # Returns
    /// A QueryBuilder with the INSERT query or an Error
    /// 
    /// 创建单条记录插入操作
    /// 
    /// # 参数
    /// * `model` - 要插入的实体模型
    /// * `primary_key` - 主键定义
    /// 
    /// # 返回值
    /// 包含 INSERT 查询的 QueryBuilder 或错误
    pub fn one(
        model: &'a ET,
        primary_key: &PrimaryKey<'a>,
    ) -> Result<QueryBuilder<'a, DB>, Error>
    {
        Self::many(once(model), primary_key)
    }

    /// 添加 RETURNING 子句
    /// 
    /// # 参数
    /// * `columns` - 要返回的列
    /// 
    /// # 返回值
    /// 更新后的构建器实例
    #[cfg(any(feature = "sqlite" , feature = "postgres"))]
    pub fn returning<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.query_builder.push(" RETURNING ");
        
        let cols = columns.into_iter();
        let mut separated = self.query_builder.separated(", ");
        for col in cols {
            separated.push(col.as_ref());
        }
        
        self
    }

    /// 添加自定义查询部分
    /// 
    /// # 参数
    /// * `build_fn` - 自定义构建函数
    /// 
    /// # 返回值
    /// 更新后的构建器实例
    pub fn custom<F>(mut self, build_fn: F) -> Self
    where
        F: FnOnce(&mut QueryBuilder<'a, DB>),
    {
        build_fn(&mut self.query_builder);
        self
    }

    /// 构建最终的查询
    /// 
    /// # 返回值
    /// QueryBuilder 实例
    pub fn finish(self) -> QueryBuilder<'a, DB> {
        self.query_builder
    }
}