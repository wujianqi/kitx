mod article;
#[cfg(any(feature = "postgres"))]
use article::Article;

#[cfg(any(feature = "postgres"))]
fn get_database_url() -> String {
    dotenv::dotenv().ok();
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

#[cfg(any(feature = "postgres"))]
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

#[cfg(feature = "postgres")]
mod postgres_tests {
    use super::*;
    use kitx::common::builder::BuilderTrait;
    use kitx::common::operations::OperationsTrait;
    use kitx::postgres::connection::init_db_pool;
    use kitx::postgres::operations::Operations;
    use kitx::postgres::sql::{field, QueryBuilder, QueryCondition};

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
        run_op(|| operations.update_one(article, false)).await;
    }

    #[tokio::test]
    async fn delete_one() {
        setup_db_pool().await;
        let operations = get_operations();
        run_op(|| operations.delete_one(1)).await;
    }

    #[tokio::test]
    async fn delete_many() {
        setup_db_pool().await;
        let operations = get_operations();
        run_op(|| operations.delete_many(vec![1, 2, 3])).await;
    }

    #[tokio::test]
    async fn fetch_all() {
        setup_db_pool().await;
        let operations = get_operations();
        run_op(|| operations.fetch_all(QueryCondition::empty())).await;
    }

    #[tokio::test]
    async fn fetch_by_key() {
        setup_db_pool().await;
        let operations = get_operations();
        run_op(|| operations.fetch_by_key(1)).await;
    }

    #[tokio::test]
    async fn fetch_one() {
        setup_db_pool().await;
        let operations = get_operations();
        let qf = QueryCondition::from(|builder: &mut QueryBuilder| {
            builder.filter(field("a_id").eq(2));
          });
        run_op(|| operations.fetch_one(qf)).await;
    }

    #[tokio::test]
    async fn fetch_by_cursor() {
        setup_db_pool().await;
        let operations = get_operations();
        let qf = QueryCondition::from(|builder: &mut QueryBuilder| {
            builder.filter(field("a_id").gt(1)).order_by("a_id", false);
          });
        run_op(|| operations.fetch_by_cursor(5, qf)).await;
    }

    #[tokio::test]
    async fn fetch_paginated() {
        setup_db_pool().await;
        let operations = get_operations();
        let qf = QueryCondition::from(|builder: &mut QueryBuilder| {
            builder.filter(field("a_id").gt(1));
          });
        run_op(|| operations.fetch_paginated(1, 5, qf)).await;
    }
}
