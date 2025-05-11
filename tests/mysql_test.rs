mod article;
#[cfg(feature = "mysql")]
use article::Article;

#[cfg(feature = "mysql")]
fn get_database_url() -> String {
    dotenv::dotenv().ok();
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[cfg(feature = "mysql")]
async fn run<F, T, E>(operation: F)
where
    F: Future<Output = Result<T, E>>,
    E: std::error::Error,
    T: std::fmt::Debug,
{
    match operation.await {
        Ok(result) => {
            dbg!(result);
            assert!(true);
        }
        Err(e) => {
            eprintln!("{:?}", e);
            assert!(false);
        }
    }
}

#[cfg(feature = "mysql")]
mod mysql_tests {
    use std::vec;

    use crate::article::ArticleTag;

    use super::*;
    use kitx::{prelude::{mysql::*, *}, utils::query_condition::empty_query};

    async fn setup_db_pool() {
        let database_url = get_database_url();
        create_db_pool(&database_url).await.unwrap();
    }

    fn sops() -> Operations<'static, Article> {
        Operations::new("article", ("id", true))
    }
    fn cops() -> MutliKeyOperations<'static, ArticleTag> {
        MutliKeyOperations::new("article_tag", vec!["article_id", "share_seq"])
    }
    
    #[tokio::test]
    async fn insert_one() {
        setup_db_pool().await;
        let mut article = Article::new(100,"test888", None);
        article.content = Some("abc".to_string());

        run(sops().insert_one(article)).await;  
    }

    #[tokio::test]
    async fn update_one() {
        setup_db_pool().await;
        let mut article = Article::new(100,"test", Some("abc123".to_string()));
        article.id = 1;

        run(sops().update_by_key(article, )).await;
    }

    #[tokio::test]
    async fn update_by_expr() {
        setup_db_pool().await;
        let columns = &[("views", "views + 1")];
        let qf = |builder: &mut Update| {
            builder.and_where_mut(Expr::col("id").eq(1));
        };

        run(sops().update_by_expr(columns, qf)).await;
    }

    #[tokio::test]
    async fn delete_one() {
        setup_db_pool().await;
        set_global_soft_delete_field("deleted", &[]);

        run(sops().delete_by_key(0)).await;
    }

    #[tokio::test]
    async fn delete_many() {
        setup_db_pool().await;

        run(sops().delete_many(vec![1, 2, 3])).await;
    }

    #[tokio::test]
    async fn get_list() {
        setup_db_pool().await;
        set_global_filter(Expr::col("tenant_id").eq(200), &[]);

        //let qf: Option<Box<dyn Fn(&mut Select) + Send>> = None;

        /* let dq = |builder: &mut Select| {
            builder.where_mut(col("a_id").gt(1));
        }; */

        let dq = empty_query();
        run(sops().get_list(dq)).await;
    }

    #[tokio::test]
    async fn get_one_by_key() {
        setup_db_pool().await;
        run(sops().get_one_by_key(1)).await;
    }

    #[tokio::test]
    async fn get_one() {
        setup_db_pool().await;
        let qf = |builder: &mut Select| {
            builder.and_where_mut(Expr::col("id").eq(2));
        };
        run(sops().get_one(qf)).await;
    }

    #[tokio::test]
    async fn get_list_by_cursor() {
        setup_db_pool().await;
        let qf = |builder: &mut Select| {
            builder.and_where_mut(Expr::col("id").gt(1)).order_by_mut("id", OrderBy::Desc);
        };
        run(sops().get_list_by_cursor(5, qf)).await;
    }

    #[tokio::test]
    async fn get_list_paginated() {
        setup_db_pool().await;
        let qf = |builder: &mut Select| {
            builder.and_where_mut(Expr::col("id").gt(1));
        };
        run(sops().get_list_paginated(1, 5, qf)).await;
    }

    #[tokio::test]
    async fn upsert_many() {
        setup_db_pool().await;

        let articles = vec![
            Article::new(200, "content1", Some("test1".to_string())),
            Article::new(200, "content2", Some("test2".to_string())),
            Article::new(200, "content10", Some("test10".to_string())),
        ];

        run(sops().upsert_many(articles)).await;
    }

    #[tokio::test]
    async fn with_relations() {
        setup_db_pool().await;
        let query = Query::shared();
        
        let article_ops = sops().set(query.share());
        let article_tag_ops = cops().set(query.share());

        let _ = query.share().begin_transaction().await;

        let mut article = Article::new(100,"test222", None);
        article.content = Some("abc".to_string());
        let mut article_ag = ArticleTag::new("tag1");
        article_ag.article_id = 1;
        article_ag.share_seq = 1234;

        let handler1 = article_ops.insert_one(article);
        let handler2 = article_tag_ops.insert_one(article_ag);

        run(handler1).await;
        run(handler2).await;
        run(query.share().commit()).await;
    }

}


#[cfg(feature = "mysql")]
mod concurrent_tests {
    use std::sync::Arc;
    use tokio::sync::Barrier;
    
    use super::*;
    use kitx::prelude::{mysql::*, *};

    async fn setup_concurrent_db_pool() {
        let database_url = get_database_url();
        create_db_pool(&database_url).await.unwrap();
    }
    fn sops() -> Arc<Operations<'static, Article>> {
        Arc::new(Operations::new("article", ("id", true)))
    }

    #[tokio::test]
    async fn concurrent_inserts() {
        setup_concurrent_db_pool().await;
        let barrier = Arc::new(Barrier::new(5));
        let ops = sops();

        for i in 0..5 {
            let barrier = Arc::clone(&barrier);
            let ops = ops.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                let mut article = Article::new(300 + i, &format!("concurrent_test{}", i), None);
                article.content = Some("concurrent_data".to_string());

                run(ops.insert_one(article)).await;
            });
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let count = ops.count(|_|{}).await.unwrap();
        assert_eq!(count, 5); 
    }

    #[tokio::test]
    async fn concurrent_updates() {
        setup_concurrent_db_pool().await;
        let barrier = Arc::new(Barrier::new(3));
        let ops = sops();

        let article = Article::new(999, "initial", Some("data".to_string()));
        let num = ops.insert_one(article).await.unwrap().last_insert_id();
        //run(ops.insert_one(article)).await;

        for _ in 0..3 {
            let barrier = Arc::clone(&barrier);
            let ops = ops.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                let qf = move |builder: &mut Update| {
                    builder.and_where_mut(Expr::col("id").eq(num));
                };
                let columns = &[("title", "'updated_by_concurrent'")];
                run(ops.update_by_expr(columns, qf)).await;
            });
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let result = ops.get_one_by_key(num).await;

        match result {
            Ok(article) => {
                if let Some(content) = article {
                    assert_eq!(content.title, "updated_by_concurrent");
                } else {
                    assert!(false, "Content should not be None");
                }                
            },
            Err(err) => {
                eprint!("Error: {}", err);
            }            
        }

    }
}
