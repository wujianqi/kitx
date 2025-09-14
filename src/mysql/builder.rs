//! MySQL query builder module
//! 
//! This module provides type aliases and utilities for building MySQL database queries.
//! It includes convenient type aliases for various query builders such as Insert, Update,
//! Delete, Select, and Upsert operations, making it easier to work with MySQL-specific
//! database operations in a type-safe manner.
//! 
//! # 中文
//! MySQL 查询构建器模块
//! 
//! 该模块提供了构建 MySQL 数据库查询的类型别名和实用工具。
//! 它包括各种查询构建器（如 Insert、Update、Delete、Select 和 Upsert 操作）的便捷类型别名，
//! 使得以类型安全的方式处理 MySQL 特定的数据库操作更加容易。

use sqlx::{QueryBuilder, MySql};

use crate::{internal::{delete_builder, insert_builder, select_builder, subquery, update_builder, upset_mysql}, mysql::kind::DataKind};

/// QueryBuilder type alias for MySQL
/// MySQL 的 QueryBuilder 类型别名
pub type QB<'a> = QueryBuilder<'a, MySql>;

/// SubqueryBuilder type alias for MySQL
/// MySQL 的 SubqueryBuilder 类型别名
pub type SQB<'a> = subquery::SubqueryBuilder<'a, DataKind>;

/// Subquery type alias for MySQL
/// MySQL 的 Subquery 类型别名
pub type Subquery<'a, ET> = subquery::Subquery<'a, ET, DataKind>;

/// Insert builder type alias for MySQL
/// MySQL 的 Insert 构建器类型别名
pub type Insert<'a, ET> = insert_builder::Insert<'a, ET, MySql, DataKind>;

/// Update builder type alias for MySQL
/// MySQL 的 Update 构建器类型别名
pub type Update<'a, ET> = update_builder::Update<'a, ET, MySql, DataKind>;

/// Delete builder type alias for MySQL
/// MySQL 的 Delete 构建器类型别名
pub type Delete<'a, ET> = delete_builder::Delete<'a, ET, MySql, DataKind>;

/// Select builder type alias for MySQL
/// MySQL 的 Select 构建器类型别名
pub type Select<'a, ET> = select_builder::Select<'a, ET, MySql, DataKind>;

/// Upsert builder type alias for MySQL
/// MySQL 的 Upsert 构建器类型别名
pub type Upset<'a, ET> = upset_mysql::Upset<'a, ET, MySql, DataKind>;

#[cfg(test)]
mod tests {

    use crate::{
        common::types::{CursorPaginatedResult, PaginatedResult, PrimaryKey, SortOrder}, 
        mysql::{builder::{Delete, Insert, Select, Subquery, Update, Upset, QB}, 
        connection, kind::DataKind, 
        query::{execute, fetch_all, fetch_one, fetch_scalar}}, 
        test_utils::{article::Article, init::get_database_url}
    };
    //use super::*;

    async fn init_pool() {
        let database_url = get_database_url().await;
        connection::create_db_pool(&database_url).await.unwrap();
    }

    static ARTICLE_KEY: PrimaryKey = PrimaryKey::Single("id", true);

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

        let qb = Upset::one(&entity, &ARTICLE_KEY).unwrap();

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

        let qb = Update::<Article>::default_table()
            .set(set_build_fn)
            .where_(filter_build_fn).unwrap();

        init_pool().await;
        let result = execute(qb).await.unwrap();
        println!("Updated {} rows.", result.rows_affected()); 
    }

    #[tokio::test]
    async fn test_delete_by_primary_key() {
        let idv = vec![1.into()];

        let qb = Delete::<Article>::by_primary_key(&ARTICLE_KEY, &idv).unwrap();

        init_pool().await;
        let result = execute(qb).await.unwrap(); 
        println!("Deleted {} rows.", result.rows_affected());
    }

    #[tokio::test]
    async fn test_delete_with_filter() {
        let filter_build_fn: fn(&mut QB) = |qb| {
            qb.push("id = ").push_bind(1 as i64);
        };
        let qb = Delete::<Article>::from_default().where_(filter_build_fn).unwrap();

        init_pool().await;
        let result = execute(qb).await.unwrap(); 
        println!("Deleted {} rows.", result.rows_affected());
    }

    #[tokio::test]
    async fn test_find_all() {
        let qb = Select::<Article>::select_default().from_default().inner();
        
        init_pool().await;
        let list = fetch_all::<Article>(qb).await.unwrap();  
        dbg!(&list);
    }

    #[tokio::test]
    async fn test_find_one() {
        let binding = vec![1.into()];
        let qb = Select::<Article>::by_primary_key(&ARTICLE_KEY, &binding).unwrap();

        init_pool().await;
        let article = fetch_one::<Article>(qb).await.unwrap();  
        dbg!(&article);
    }

    #[tokio::test]
    async fn test_nested_subquery() {
        init_pool().await;
        let avg_views_subquery = Subquery::<Article>::select(|b| {
                b.push("AVG(views)");
            })
            .from_default()
            .where_(|b| {
                b.push("id > ").push_bind(3.into());
            });

        let qb = Select::<Article>::select_default()
            .from_default()
            .where_(move |b| {
                b.push("views <");
                avg_views_subquery.append_to(b);
            })
            .inner();

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

        let qb = Select::<Article>::select_default()
            .from_default()
            .where_(filter_build_fn)
            .order_by("id DESC")
            .paginate(1, 10)
            .unwrap();
        
        init_pool().await;
        let list = fetch_all::<Article>(qb).await.unwrap();

        let column_build_fn: fn(&mut QB) = |qb| {
            qb.push("count(id)");
        };
        let qb2 = Select::<Article>::select(column_build_fn)
            .from_default()
            .where_(filter_build_fn)
            .inner();
        
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
        let sort_order = SortOrder::Asc;
        let column_key = "id";

        // 初始请求（无游标）
        let cursor_qb = Select::<Article>::select_default()
            .from_default()
            .cursor::<DataKind>(column_key, &sort_order, None, limit)
            .unwrap();
        
        let result1 = fetch_all::<Article>(cursor_qb).await.unwrap();
        let mut paginated1 = CursorPaginatedResult::new(result1, limit, sort_order.clone());
        paginated1.gen_cursors(column_key);

        dbg!(&paginated1);
        
        // 使用next_cursor获取下一页
        let next_cursor = paginated1.next_cursor;
        let cursor_qb2 = Select::<Article>::select_default()
            .from_default()
            .cursor::<DataKind>(column_key, &sort_order, next_cursor, limit)
            .unwrap();
        
        let result2 = fetch_all::<Article>(cursor_qb2).await.unwrap();
        let mut paginated2 = CursorPaginatedResult::<Article, DataKind>::new(result2, limit, sort_order.clone());
        paginated2.gen_cursors(column_key);
        
        dbg!(&paginated2);
        
        // 验证排序逻辑（降序测试）
        let cursor_qb_desc = Select::<Article>::select_default()
            .from_default()
            .cursor::<DataKind>(column_key, &SortOrder::Desc, None, limit)
            .unwrap();
        
        let result_desc = fetch_all::<Article>(cursor_qb_desc).await.unwrap();
        let mut paginated_desc = CursorPaginatedResult::<Article, DataKind>::new(result_desc, limit, SortOrder::Desc);
        paginated_desc.gen_cursors("id");
        
        dbg!(&paginated_desc);
    }

}