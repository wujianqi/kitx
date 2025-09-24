//! MySQL query builder module
//! 
//! This module provides type aliases and utilities for building MySQL database queries.
//! It includes convenient type aliases for various query builders such as Insert, Update,
//! Delete, Select, and Upsert operations, making it easier to work with MySQL-specific
//! database operations in a type-safe manner.
//! 
//! MySQL 查询构建器模块
//! 
//! 该模块提供了构建 MySQL 数据库查询的类型别名和实用工具。
//! 它包括各种查询构建器（如 Insert、Update、Delete、Select 和 Upsert 操作）的便捷类型别名，
//! 使得以类型安全的方式处理 MySQL 特定的数据库操作更加容易。

use sqlx::{QueryBuilder, MySql};

use crate::{internal::{delete_builder, insert_builder, select_builder, subquery, update_builder, upsert_mysql}, mysql::kind::DataKind};

/// QueryBuilder type alias for MySQL  
/// MySQL 的 QueryBuilder 类型别名
/// 
/// This is a type alias for sqlx::QueryBuilder<'a, MySql>, which is used to build dynamic SQL queries
/// in a type-safe manner for MySQL databases. It provides methods to push SQL fragments, bind parameters,
/// and construct complex queries programmatically.
/// 
/// 这是 sqlx::QueryBuilder<'a, MySql> 的类型别名，用于以类型安全的方式为 MySQL 数据库构建动态 SQL 查询。
/// 它提供了推送 SQL 片段、绑定参数和以编程方式构造复杂查询的方法。
/// 
/// # Examples
/// 
/// ```
/// use kitx::mysql::builder::QB;
/// 
/// let mut query_builder: QB = QB::new("SELECT * FROM users");
/// query_builder.push(" WHERE id = ").push_bind(1);
/// ```
/// 
/// # Public Methods
/// 
/// Refer to [sqlx::QueryBuilder](https://docs.rs/sqlx/latest/sqlx/struct.QueryBuilder.html) for all available methods.
/// 
/// # 公共方法
/// 
/// 请参考 [sqlx::QueryBuilder](https://docs.rs/sqlx/latest/sqlx/struct.QueryBuilder.html) 获取所有可用方法。
pub type QB<'a> = QueryBuilder<'a, MySql>;

/// SubqueryBuilder type alias for MySQL  
/// MySQL 的 SubqueryBuilder 类型别名
/// 
/// This type is used to build subqueries that can be embedded within other queries. It provides
/// a way to construct complex nested queries in a type-safe manner.
/// 
/// 该类型用于构建可以嵌入到其他查询中的子查询。它提供了一种以类型安全的方式构造复杂嵌套查询的方法。
/// 
/// # Public Methods
/// 
/// * `push` - Add a text fragment to the subquery
/// * `push_bind` - Add a binding value to the subquery
/// 
/// # 公共方法
/// 
/// * `push` - 向子查询中添加文本片段
/// * `push_bind` - 向子查询中添加绑定值
/// 
/// # Examples
/// 
/// ```
/// use kitx::mysql::builder::{SQB, Subquery};
/// 
/// let subquery = Subquery::with_table("users")
///     .columns(|b| b.push("*"))  
///     .filter(|b| b.push("age > ").push_bind(18));
/// ```
pub type SQB<'a> = subquery::SubqueryBuilder<'a, DataKind>;

/// Subquery type alias for MySQL  
/// MySQL 的 Subquery 类型别名
/// 
/// A complete subquery that can be appended to other queries. This type represents a fully constructed
/// subquery that can be embedded in WHERE clauses, FROM clauses, or other parts of SQL statements.
/// 
/// 可以附加到其他查询的完整子查询。此类型表示可以嵌入到 WHERE 子句、FROM 子句或其他 SQL 语句部分中的完全构造的子查询。
/// 
/// # Type Parameters
/// 
/// * `ET` - The entity type that this subquery operates on
/// * `ET` - 此子查询操作的实体类型
/// 
/// # Public Methods
/// 
/// * `table` - Create a subquery with the default table name
/// * `with_table` - Create a subquery with a custom table name
/// * `columns` - Add the default table to the subquery
/// * `filter` - Add WHERE condition to the subquery
/// * `join` - Add JOIN clause to the subquery
/// * `group_by` -  Add GROUP BY clause to the subquery
/// * `having` - Add HAVING clause to the subquery
/// * `append_to` - Embed the subquery into a parent query builder
/// 
/// # 公共方法
/// 
/// * `table` - 创建带有默认表名的子查询
/// * `with_table` - 创建带有自定义表名的子查询
/// * `columns` - 向子查询中添加自定义表
/// * `filter` - 向子查询中添加 WHERE 条件
/// * `join` - 向子查询中添加 JOIN 子句
/// * `group_by` - 向子查询中添加 GROUP BY 子句
/// * `having` -  向子查询中添加 HAVING 子句
/// * `append_to` - 将子查询嵌入到父查询构建器中
/// 
/// # Examples
/// 
/// ```
/// use kitx::mysql::builder::{Subquery, Select};
/// 
/// let subquery: Subquery<Article> = Subquery::table()
///     .columns(|b| {
///         b.push("AVG(views)");
///     })
///     .filter(|b| {
///         b.push("id > ").push_bind(3);
///     });
/// ```
pub type Subquery<'a, ET> = subquery::Subquery<'a, ET, DataKind>;

/// Insert builder type alias for MySQL  
/// MySQL 的 Insert 构建器类型别名
/// 
/// Used to build INSERT statements for MySQL databases. Provides methods to insert single or multiple
/// records in a type-safe manner.
/// 
/// 用于为 MySQL 数据库构建 INSERT 语句。提供以类型安全的方式插入单个或多个记录的方法。
/// 
/// # Type Parameters
/// 
/// * `ET` - The entity type that this insert builder operates on
/// * `ET` - 此插入构建器操作的实体类型
/// 
/// # Public Methods
/// 
/// * `one` - Create single record insert operation
/// * `many` - Create multiple records insert operation
/// * `table` - Create custom table and columns
/// * `with_table` - Create a insert with a custom table name
/// * `from_query` - Create an Insert instance from a query
/// * `from_query_with_table` - Create an Insert instance from a query with a custom table name
/// * `custom` - Custom VALUES or value-related query statements
/// * `finish` - Finish building, get the internal QueryBuilder
/// 
/// # 公共方法
/// 
/// * `one` - 创建单条记录插入操作
/// * `many` - 创建多条记录插入操作
/// * `table` - 创建默认表名的插入操作
/// * `with_table` - 创建带有自定义表名的插入操作
/// * `from_query` - 从外部查询中创建 Insert 实例
/// * `from_query_with_table` - 从外部查询中创建 Insert 实例，可以自定义表名
/// * `custom` - 自定义 VALUES 或值相关的查询语句
/// * `finish` - 结束构建，获取内部的 QueryBuilder
/// 
/// # Examples
/// 
/// ```
/// use kitx::mysql::builder::Insert;
/// 
/// let entity = User { id: 1, name: "Alice".to_string() };
/// let insert_query = Insert::one(&entity, &PRIMARY_KEY).unwrap();
/// ```
pub type Insert<'a, ET> = insert_builder::Insert<'a, ET, MySql, DataKind>;

/// Update builder type alias for MySQL  
/// MySQL 的 Update 构建器类型别名
/// 
/// Used to build UPDATE statements for MySQL databases. Provides methods to update records with
/// flexible WHERE clauses and SET operations.
/// 
/// 用于为 MySQL 数据库构建 UPDATE 语句。提供使用灵活的 WHERE 子句和 SET 操作更新记录的方法。
/// 
/// # Type Parameters
/// 
/// * `ET` - The entity type that this update builder operates on
/// * `ET` - 此更新构建器操作的实体类型
/// 
/// # Public Methods
/// 
/// * `one` - Create a single entity update operation
/// * `table` - Create an Update instance with the default table name
/// * `with_table` - Create an Update instance with a custom table name
/// * `from_query` - Create an Update instance from a query
/// * `from_query_with_table` - Create an Update instance from a query with a custom table name
/// * `custom` - Custom SET columns or other query statements
/// * `filter` - Add WHERE condition to the update statement
/// * `finish` - Finish building, get the internal QueryBuilder
/// 
/// # 公共方法
/// 
/// * `one` - 创建单个实体更新操作
/// * `table` - 创建使用默认表名的 Update 实例
/// * `with_table` - 创建使用自定义表名的 Update 实例
/// * `from_query` - 从外部查询中创建 Update 实例
/// * `from_query_with_table` - 从外部查询中创建 Update 实例，可以自定义表名
/// * `custom` - 自定义 SET 列或其他查询语句
/// * `filter` - 向查询中添加过滤查询部分
/// * `finish` - 结束构建，获取内部的 QueryBuilder
/// 
/// # Examples
/// 
/// ```
/// use kitx::mysql::builder::Update;
/// 
/// let entity = User { id: 1, name: "Bob".to_string() };
/// let update_query = Update::one(&entity, &PRIMARY_KEY, true).unwrap();
/// ```
pub type Update<'a, ET> = update_builder::Update<'a, ET, MySql, DataKind>;

/// Delete builder type alias for MySQL  
/// MySQL 的 Delete 构建器类型别名
/// 
/// Used to build DELETE statements for MySQL databases. Provides methods to delete records by
/// primary key or with custom WHERE clauses.
/// 
/// 用于为 MySQL 数据库构建 DELETE 语句。提供按主键或使用自定义 WHERE 子句删除记录的方法。
/// 
/// # Type Parameters
/// 
/// * `ET` - The entity type that this delete builder operates on
/// * `ET` - 此删除构建器操作的实体类型
/// 
/// # Public Methods
/// 
/// * `table` - Create a Delete instance using the default table name
/// * `with_table` - Create a Delete instance with a custom table name
/// * `from_query` - Create an Delete instance from a query
/// * `from_query_with_table` - Create an Delete instance from a query with a custom table name
/// * `by_primary_key` - Create a DELETE query by primary key
/// * `filter` - Create a DELETE query with custom WHERE conditions
/// * `finish` - Finish building, get the internal QueryBuilder
/// 
/// # 公共方法
/// 
/// * `table` - 创建使用默认表名的 Delete 实例
/// * `with_table` - 使用自定义表名创建 Delete 实例
/// * `from_query` - 从外部查询中创建 Delete 实例
/// * `from_query_with_table` - 从外部查询中创建 Delete 实例，可以自定义表名
/// * `by_primary_key` - 通过主键创建 DELETE 查询
/// * `filter` - 创建带有自定义 WHERE 条件的 DELETE 查询
/// * `finish` - 结束构建，获取内部的 QueryBuilder
/// 
/// # Examples
/// 
/// ```
/// use kitx::mysql::builder::Delete;
/// 
/// let ids = vec![1.into(), 2.into()];
/// let delete_query = Delete::<User>::table().by_primary_key(&PRIMARY_KEY, &ids).finish();
/// ```
pub type Delete<'a, ET> = delete_builder::Delete<'a, ET, MySql, DataKind>;

/// Select builder type alias for MySQL  
/// MySQL 的 Select 构建器类型别名
/// 
/// Used to build SELECT statements for MySQL databases. Provides methods to construct queries
/// with flexible WHERE clauses, JOINs, ORDER BY, LIMIT and other SQL features.
/// 
/// 用于为 MySQL 数据库构建 SELECT 语句。提供构造具有灵活 WHERE 子句、JOIN、ORDER BY、LIMIT 和其他 SQL 特性的查询的方法。
/// 
/// # Type Parameters
/// 
/// * `ET` - The entity type that this select builder operates on
/// * `ET` - 此选择构建器操作的实体类型
/// 
/// # Public Methods
/// 
/// * `table` - Create a Select instance using the default table name 
/// * `with_table` - Create a Select instance with a custom table name
/// * `from_query` - Create an Select instance from a query
/// * `from_query_with_table` - Create an Select instance from a query with a custom table name
/// * `columns` - Create a custom column query statement
/// * `filter` - Create a SELECT query with custom WHERE conditions
/// * `join` - Create a JOIN query statement
/// * `group_by` - Create a GROUP BY query statement
/// * `having` - Create a HAVING clause
/// * `by_primary_key` - Create a SELECT query by primary key
/// * `order_by` - Create an ORDER BY clause
/// * `paginate` - Create a pagination query statement
/// * `cursor` - Create a cursor pagination query statement
/// * `finish` - Finish building, get the internal QueryBuilder
/// 
/// # 公共方法
/// 
/// * `table` - 创建使用默认表名的 Select 实例
/// * `with_table` - 创建使用自定义表名的 Select 实例
/// * `from_query` - 从外部查询中创建 Select 实例
/// * `from_query_with_table` - 从外部查询中创建 Select 实例，可以自定义表名
/// * `columns` - 创建自定义列的查询语句
/// * `filter` - 创建带有自定义 WHERE 条件的查询语句
/// * `join` - 创建 JOIN 查询语句
/// * `group_by` - 创建 GROUP BY 查询语句 
/// * `having` - 创建 HAVING 子句
/// * `by_primary_key` - 创建按主键条件查询语句
/// * `order_by` - 创建排序子句
/// * `paginate` - 创建分页查询语句
/// * `cursor` - 创建游标分页查询语句
/// * `finish` - 结束构建，获取内部的 QueryBuilder
/// 
/// # Examples
/// 
/// ```
/// use kitx::mysql::builder::Select;
/// 
/// let select_query = Select::<User>::table().finish();
/// ```
pub type Select<'a, ET> = select_builder::Select<'a, ET, MySql, DataKind>;

/// Upsert builder type alias for MySQL  
/// MySQL 的 Upsert 构建器类型别名
/// 
/// Used to build UPSERT statements for MySQL databases (using INSERT ... ON DUPLICATE KEY UPDATE syntax).
/// Provides methods to insert records or replace them if they already exist based on primary key.
/// 
/// 用于为 MySQL 数据库构建 UPSERT 语句（使用 INSERT ... ON DUPLICATE KEY UPDATE 语法）。
/// 提供插入记录或在记录已存在时基于主键替换它们的方法。
/// 
/// # Type Parameters
/// 
/// * `ET` - The entity type that this upsert builder operates on
/// * `ET` - 此更新插入构建器操作的实体类型
/// 
/// # Public Methods
/// 
/// * `one` - Create single record upsert operation
/// * `many` - Create multiple records upsert operation
/// 
/// # 公共方法
/// 
/// * `one` - 创建单条记录更新插入操作
/// * `many` - 创建多条记录更新插入操作
/// 
/// # Examples
/// 
/// ```
/// use kitx::mysql::builder::Upsert;
/// 
/// let entity = User { id: 1, name: "Charlie".to_string() };
/// let upsert_query = Upsert::one(&entity, &PRIMARY_KEY).unwrap();
/// ```
pub type Upsert<'a, ET> = upsert_mysql::Upsert<'a, ET, MySql, DataKind>;

#[cfg(test)]
mod tests {
    use crate::{
        common::types::{CursorPaginatedResult, PaginatedResult, PrimaryKey, Order}, 
        mysql::{builder::{Delete, Insert, Select, Subquery, Update, Upsert, QB}, 
        connection, kind::DataKind, 
        query::{execute, fetch_all, fetch_one, fetch_scalar}}, 
        test_utils::{article::Article, init::get_database_url}
    };
    //use super::*;
    async fn init_pool() {
        let database_url = get_database_url().await;
        connection::create_db_pool(&database_url).await.unwrap();
    }

    const ARTICLE_KEY: PrimaryKey = PrimaryKey::Single("id", true);

    #[tokio::test]
    async fn test_insert_one() {
        let mut entity = Article::new(100,"vvvv", None);
        entity.content = Some("abc".to_string());

        let qb = Insert::one(&entity, &ARTICLE_KEY).unwrap();

        init_pool().await;
        let result = execute(qb).await.unwrap(); 
        println!("Inserted {} rows.", result.rows_affected());
    }

    #[tokio::test]
    async fn test_insert_many() {
        let mut entity1 = Article::new(100,"t111", None);
        entity1.content = Some("abc111".to_string());
        let mut entity2 = Article::new(100,"t2222", None);
        entity2.content = Some("abc222".to_string());

        let binding = [entity1, entity2];
        let qb = Insert::many(&binding, &ARTICLE_KEY).unwrap();

        init_pool().await;
        let result = execute(qb).await.unwrap(); 
        println!("Inserted {} rows.", result.rows_affected());
    }

    #[tokio::test]
    async fn test_upset_one() {
        let mut entity = Article::new(100,"t1", None);
        entity.content = Some("abc".to_string());
        entity.id = 0;

        let qb = Upsert::one(&entity, &ARTICLE_KEY).unwrap();

        init_pool().await;
        let result = execute(qb).await.unwrap(); 
        assert_eq!(result.rows_affected(), 1);
    }

    #[tokio::test]
    async fn test_update_one() {
        let mut entity = Article::new(110,"test9999", None);
        entity.content = Some("abc111".to_string());
        entity.id = 1;

        let qb = Update::one(&entity, &ARTICLE_KEY, true).unwrap();

        init_pool().await;
        let result = execute(qb).await.unwrap(); 
        println!("Updated {} rows.", result.rows_affected());
    }

    #[tokio::test]
    async fn test_update_with_filter() {
        let set_build_fn: fn(&mut QB) = |qb| {
            qb.push("views = views + 1");
        };

        let filter_build_fn: fn(&mut QB) = |qb| {
            qb.push("id = ").push_bind(1 as i64);
        };

        let qb = Update::<Article>::table()
            .custom(set_build_fn)
            .filter(filter_build_fn)
            .finish();

        init_pool().await;
        let result = execute(qb).await.unwrap();
        println!("Updated {} rows.", result.rows_affected()); 
    }

    #[tokio::test]
    async fn test_delete_by_primary_key() {
        let idv = vec![1.into()];

        let qb = Delete::<Article>::table()
            .by_primary_key(&ARTICLE_KEY, &idv)
            .finish();

        init_pool().await;
        let result = execute(qb).await.unwrap(); 
        println!("Deleted {} rows.", result.rows_affected());
    }

    #[tokio::test]
    async fn test_delete_with_filter() {
        let filter_build_fn: fn(&mut QB) = |qb| {
            qb.push("id = ").push_bind(1 as i64);
        };
        let qb = Delete::<Article>::table()
            .filter(filter_build_fn)
            .finish();

        init_pool().await;
        let result = execute(qb).await.unwrap(); 
        println!("Deleted {} rows.", result.rows_affected());
    }

    #[tokio::test]
    async fn test_find_all() {
        let qb = Select::<Article>::table().finish();
        
        init_pool().await;
        let list = fetch_all::<Article>(qb).await.unwrap();  
        dbg!(&list);
    }

    #[tokio::test]
    async fn test_find_one() {
        let binding = vec![1.into()];
        let qb = Select::<Article>::table()
            .by_primary_key(&ARTICLE_KEY, &binding)
            .finish();

        init_pool().await;
        let article = fetch_one::<Article>(qb).await.unwrap();  
        dbg!(&article);
    }

    #[tokio::test]
    async fn test_nested_subquery() {
        init_pool().await;
        let avg_views_subquery = Subquery::<Article>::table()
            .columns(|b| {
                b.push("AVG(views)");
            })
            .filter(|b| {
                b.push("id > ").push_bind(3.into());
            });

        let qb = Select::<Article>::table()
            .filter(move |b| {
                b.push("views <");
                avg_views_subquery.append_to(b);
            })
            .finish();

        let result = fetch_all::<Article>(qb).await.unwrap();
        dbg!(&result);
        //assert_eq!(result.len(), 1);
        //assert_eq!(result[0].views, 150);
    }

    #[tokio::test]
    async fn test_find_list_paginated() {
        let filter_build_fn = |qb: &mut QB| {
            qb.push("id > ").push_bind(1 as i64);
        };

        let qb = Select::<Article>::table()
            .filter(filter_build_fn)
            .order_by("id", Order::Desc)
            .paginate(1, 10).unwrap();
        
        init_pool().await;
        let list = fetch_all::<Article>(qb).await.unwrap();

        let qb2 = Select::<Article>::table()
            .columns(|b| { 
                b.push("count(id)"); 
            })
            .filter(filter_build_fn)
            .finish();
        
        let total = fetch_scalar(qb2).await.unwrap() as u64;

        let pr = PaginatedResult::new(list, total, 1, 10);
        dbg!(pr);
    }

    #[tokio::test]
    async fn test_find_list_by_cursor() {
        // 初始化连接池
        init_pool().await;

        // 测试参数
        let limit = 2;
        let column_key = "id";

        // 初始请求（无游标）
        let cursor_qb = Select::<Article>::table()
            .cursor(column_key, Order::Asc, None, limit).unwrap();
        
        let result1 = fetch_all::<Article>(cursor_qb).await.unwrap();
        let mut paginated1 = CursorPaginatedResult::new(result1, limit, Order::Asc);
        paginated1.gen_cursors(column_key);

        dbg!(&paginated1);
        
        // 使用next_cursor获取下一页
        let next_cursor = paginated1.next_cursor;
        let cursor_qb2 = Select::<Article>::table()
            .cursor(column_key, Order::Asc, next_cursor, limit).unwrap();
        
        let result2 = fetch_all::<Article>(cursor_qb2).await.unwrap();
        let mut paginated2 = CursorPaginatedResult::<Article, DataKind>::new(result2, limit, Order::Asc);
        paginated2.gen_cursors(column_key);
        
        dbg!(&paginated2);
        
        // 验证排序逻辑（降序测试）
        let cursor_qb_desc = Select::<Article>::table()
            .cursor(column_key, Order::Desc, None, limit).unwrap();
        
        let result_desc = fetch_all::<Article>(cursor_qb_desc).await.unwrap();
        let mut paginated_desc = CursorPaginatedResult::<Article, DataKind>::new(result_desc, limit, Order::Desc);
        paginated_desc.gen_cursors("id");
        
        dbg!(&paginated_desc);
    }

    #[tokio::test]
    async fn test_with_cte() {
        init_pool().await;

        let mut cte_builder = QB::new("WITH article_cte AS ");
        Subquery::<Article>::table()            
            .filter( |b| {
                b.push("id > ").push_bind(50.into());
            })
            .append_to(&mut cte_builder);

        let qb = Select::<Article>::from_query_with_table(cte_builder, "article_cte")
            .finish();
        
        // 执行查询
        let result = fetch_all::<Article>(qb).await.unwrap();
        
        // 添加验证断言
        assert!(!result.is_empty(), "CTE查询结果不应为空");
        assert!(result.len() > 0, "应返回至少一条记录");
        
        dbg!(&result);
    }

}