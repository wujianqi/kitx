mod article;
#[cfg(feature = "postgres")]
use article::Article;

#[cfg(feature = "postgres")]
fn get_database_url() -> String {
    dotenv::dotenv().ok();
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[cfg(feature = "postgres")]
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

#[cfg(feature = "postgres")]
mod postgres_tests {
    use std::vec;

    use crate::article::ArticleTag;

    use super::*;
    use kitx::{prelude::{postgres::*, *}, utils::query_condition::empty_query};

    async fn setup_db_pool() {
        let database_url = get_database_url();
        create_db_pool(&database_url).await.unwrap();
    }

    fn sops() -> Operations<'static, Article> {
        Operations::<Article>::new("article", ("id", true))
    }
    fn cops() -> MutliKeyOperations<'static, ArticleTag> {
        MutliKeyOperations::<ArticleTag>::new("article_tag", vec!["article_id", "share_seq"])
    }
    
    #[tokio::test]
    async fn insert_one() {
        setup_db_pool().await;
        let mut article = Article::new(100,"test888", None);
        article.content = Some("abc".to_string());

        run(sops().insert_one(article)).await;  
    }

    #[tokio::test]
    async fn insert_many() {
        setup_db_pool().await;
        let mut article1 = Article::new(100,"test111", None);
        article1.content = Some("m1".to_string());

        let mut article2 = Article::new(100,"test222", None);
        article2.content = Some("m2".to_string());

        run(sops().insert_many(vec![article1, article2])).await;  
    }

    #[tokio::test]
    async fn update_one() {
        setup_db_pool().await;
        let mut article = Article::new(100,"test", Some("abc123".to_string()));
        article.id = 1;

        run(sops().update_one(article)).await;
    }

    #[tokio::test]
    async fn update_by_expr() {
        setup_db_pool().await;

        let qf = |builder: &mut Update| {
            builder.set_expr_mut("views", "views + 1")
                .and_where_mut(Expr::col("id").eq(1));
        };

        run(sops().update_by_cond(qf)).await;
    }

    #[tokio::test]
    async fn upsert_many() {
        setup_db_pool().await;
        let mut article1 = Article::new(100,"testbbbb", None);
        article1.content = Some("upsert1".to_string());
        article1.id = 1;
        let mut article2 = Article::new(100,"testaaaa", None);
        article2.content = Some("upsert2".to_string());
        article2.id = 2;

        let articles = vec![
            article1,
            article2,
            Article::new(200, "contenttest22", Some("testcccc".to_string())),
        ];

        run(sops().upsert_many(articles)).await;
    }

    #[tokio::test]
    async fn delete_one() {
        setup_db_pool().await;
        set_global_soft_delete_field("deleted", &[]);

        run(sops().delete_by_pk(DataKind::from(1))).await;
    }

    #[tokio::test]
    async fn delete_many() {
        setup_db_pool().await;
        let qf = |builder: &mut Delete| {
            builder.and_where_mut(Expr::col("id").is_in(vec![1, 2, 3]));
        };

        run(sops().delete_by_cond(qf)).await;
    }

    #[tokio::test]
    async fn get_list() {
        setup_db_pool().await;
        //set_global_filter(Expr::col("tenant_id").eq(200), &[]);

        //let qf: Option<Box<dyn Fn(&mut Select) + Send>> = None;

        /* let dq = |builder: &mut Select| {
            builder.where_mut(col("a_id").gt(1));
        }; */

        let dq = empty_query();
        run(sops().get_list_by_cond(dq)).await;
    }

    #[tokio::test]
    async fn get_one_by_key() {
        setup_db_pool().await;
        run(sops().get_one_by_pk(DataKind::from(1))).await;
    }

    #[tokio::test]
    async fn get_one() {
        setup_db_pool().await;
        let qf = |builder: &mut Select| {
            builder.and_where_mut(Expr::col("id").eq(2));
        };
        run(sops().get_one_by_cond(qf)).await;
    }

    #[tokio::test]
    async fn get_list_by_cursor() {
        setup_db_pool().await;
        let qf = |builder: &mut Select| {
            builder.and_where_mut(Expr::col("id").gt(1)).order_by_mut("id", OrderBy::Desc);
        };
        run(sops().get_list_by_cursor(5, qf, |article| article.id)).await;
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
    async fn with_relations_find_one() {
        setup_db_pool().await;

        let qf = |builder: &mut Select| {
            builder.alias_mut("tag")
                .and_where_mut(Expr::col("article_id").eq(1))
                .join_mut(JoinType::inner("article")
                    .on(Expr::from_str("tag.article_id = article.id")));
        };
        run(cops().get_one_by_cond(qf)).await;
        
    }

    #[tokio::test]
    async fn with_relations_create() {
        setup_db_pool().await;
        let query = Query::shared();
        
        let article_ops = sops().set(query.share());
        let article_tag_ops = cops().set(query.share());

        let _ = query.share().begin_transaction().await;

        let mut article = Article::new(100,"test222", None);
        article.content = Some("abc".to_string());
        let mut article_tag = ArticleTag::new("tag1");
        article_tag.article_id = 1;
        article_tag.share_seq = 1234;

        let ev = EntitiesRelation::one_to_one(&article.id)
            .validate(vec![&article_tag.article_id]);

        match ev {
            Ok(_) => {
                let handler1 = article_ops.insert_one(article);
                let handler2 = article_tag_ops.insert_one(article_tag);

                run(handler1).await;
                run(handler2).await;
                run(query.share().commit()).await;
            }
            Err(e) => {
                eprintln!("{:?}", e);
                assert!(false);
            }
        }
        
    }

}


#[cfg(feature = "sqlite")]
mod concurrent_tests {
    use std::sync::Arc;
    use tokio::sync::Barrier;
    
    use super::*;
    use kitx::prelude::{sqlite::*, *};

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
        let num = ops.insert_one(article).await.unwrap().last_insert_rowid();
        //run(ops.insert_one(article)).await;

        for _ in 0..3 {
            let barrier = Arc::clone(&barrier);
            let ops = ops.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                let qf = move |builder: &mut Update| {
                    builder.and_where_mut(Expr::col("id").eq(num))
                        .set_expr_mut("title", "'updated_by_concurrent'");
                };
                run(ops.update_by_cond(qf)).await;
            });
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let result = ops.get_one_by_pk(DataKind::from(num)).await;

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
