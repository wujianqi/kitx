mod article;
#[cfg(any(feature = "sqlite"))]
use article::Article;

#[cfg(any(feature = "sqlite"))]
fn get_database_url() -> String {
    dotenv::dotenv().ok();
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[cfg(any(feature = "sqlite"))]
async fn run_op<F, T, E>(operation: F)
where
    F: AsyncFnOnce() -> Result<T, E>,
    E: std::error::Error + 'static,
    T: std::fmt::Debug,
{
    match operation().await {
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

#[cfg(feature = "sqlite")]
mod sqlite_tests {
    use super::*;
    use kitx::{
        common::{
            builder::{FilterTrait, QueryTrait}, 
            operations::OperationsTrait, 
            util::{dyn_query, empty_query}
        }, 
        sqlite::{
            connection::init_db_pool, 
            operations::Operations, 
            sql::{col, Select}
        }
    };

    async fn setup_db_pool() {
        let database_url = get_database_url();
        init_db_pool(&database_url).await.unwrap();
    }

    fn get_operations() -> Operations<'static, Article> {
        Operations::new("article", ("a_id", true))
    }
    
    #[tokio::test]
    async fn insert_one() {
        setup_db_pool().await;
        let article = Article::new("test","test3", None);
        let operations = get_operations();
        run_op(|| operations.insert_one(article)).await;  
    }

    #[tokio::test]
    async fn update_one() {
        setup_db_pool().await;
        let article = Article::new("test","test3", Some(1));
        let operations = get_operations();
        run_op(|| operations.update_by_key(article, )).await;
    }

    #[tokio::test]
    async fn delete_one() {
        setup_db_pool().await;
        let operations = get_operations();
        run_op(|| operations.delete_by_key(1)).await;
    }

    #[tokio::test]
    async fn delete_many() {
        setup_db_pool().await;
        let operations = get_operations();
        run_op(|| operations.delete_many(vec![1, 2, 3])).await;
    }

    #[tokio::test]
    async fn get_list() {
        setup_db_pool().await;
        let operations = get_operations();
        //let qf: Option<Box<dyn Fn(&mut Select) + Send>> = None;

        /* let dq = dyn_query(|builder: &mut Select| {
            builder.where_mut(col("a_id").gt(1));
        }); */

        let dq = empty_query();

        run_op(|| operations.get_list(dq)).await;
    }

    #[tokio::test]
    async fn get_by_key() {
        setup_db_pool().await;
        let operations = get_operations();
        run_op(|| operations.get_by_key(1)).await;
    }

    #[tokio::test]
    async fn get_one() {
        setup_db_pool().await;
        let operations = get_operations();
        let qf = dyn_query(|builder: &mut Select| {
            builder.where_mut(col("a_id").eq(2));
          });
        run_op(|| operations.get_one(qf)).await;
    }

    #[tokio::test]
    async fn get_list_by_cursor() {
        setup_db_pool().await;
        let operations = get_operations();
        let qf = Some(|builder: &mut Select| {
            builder.where_mut(col("a_id").gt(1)).order_by_mut("a_id", false);
          });
        run_op(|| operations.get_list_by_cursor(5, qf)).await;
    }

    #[tokio::test]
    async fn get_list_paginated() {
        setup_db_pool().await;
        let operations = get_operations();
        let qf = Some(|builder: &mut Select| {
            builder.where_mut(col("a_id").gt(1));
          });
        run_op(|| operations.get_list_paginated(1, 5, qf)).await;
    }

    #[tokio::test]
    async fn upsert_many() {
        setup_db_pool().await;
        let operations = get_operations();

        let articles = vec![
            Article::new("title1", "content1", Some(1)),
            Article::new("title2", "content2", Some(2)),
            Article::new("title3", "content21", Some(21)),
        ];

        run_op(|| operations.upsert_many(articles)).await;
    }
}
