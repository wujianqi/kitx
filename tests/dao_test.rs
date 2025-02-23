use kitx::common::builder::BuilderTrait;
use kitx::common::operations::OperationsTrait;
//use kitx::sqlite::sql::{field, QueryBuilder, QueryCondition};
use kitx::mysql::sql::{field, QueryBuilder, QueryCondition};

use kitx::{
  //sqlite::connection::init_db_pool,
  mysql::connection::init_db_pool
};

mod article;
use article::Article;
use article::ArticleService;

async fn setup_db_pool() {
  //init_db_pool("sqlite:./my.db").await.unwrap();
  init_db_pool("mysql://root:@localhost:3306/kitxtest?charset=utf8mb4").await.unwrap();
}

#[tokio::test]
async fn insert() {
    setup_db_pool().await;

    let article = Article {
      a_class: Some("其他2".to_string()),
      a_content: Some("真测试test".to_string()),
      ..Default::default()
    };
    let ase = ArticleService::new();
    let result = ase.insert_one(article).await;

    match result {
        Ok(ret) => {
          dbg!(ret);
          assert!(true);
        },
        Err(e) =>{          
          eprintln!("插入失败: {:?}", e);
          assert!(false);
        }
    }
}


#[tokio::test]
async fn update() {
    setup_db_pool().await;

    let article = Article {
      a_id: 1,
      a_class: Some("关于我们".to_string()),
      a_content: Some("测试修改内容33".to_string()),
    };
    let ase = ArticleService::new();
    let result = ase.update_one(article, false).await;

    match result {
        Ok(ret) => {
          dbg!(ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("更新失败: {:?}", e);
          assert!(false);
        }
    }
}

#[tokio::test]
async fn delete() {
    setup_db_pool().await;

    let ase = ArticleService::new();
    let result = ase.delete_one(5).await;

    match result {
        Ok(ret) => {
          dbg!(ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("删除失败: {:?}", e);
          assert!(false);
        }
    }
}


#[tokio::test]
async fn batch_delete() {
    setup_db_pool().await;

    let ase = ArticleService::new();
    let result = ase.delete_many(vec![2, 3, 6]).await;

    match result {
        Ok(ret) => {
          dbg!(ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("删除失败: {:?}", e);
          assert!(false);
        }
    }
}


#[tokio::test]
async fn get_list() {
    setup_db_pool().await;

    let ase = ArticleService::new();
    /* let result = ase.fetch_paginated(1, 10, QueryCondition::empty()).await;
 */
    let qf = QueryCondition::from(|builder: &mut QueryBuilder| {
      builder.filter(field("a_id").gt(1));
    });
    //let result = ase.fetch_paginated(1, 10, qf).await;
    let result = ase.fetch_by_cursor(2, qf).await;

    match result {
        Ok(ret) => {
          dbg!(ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("获取失败: {:?}", e);
          assert!(false);
        }
    }
}


#[tokio::test]
async fn get_by_key() {
    setup_db_pool().await;

    let ase = ArticleService::new();
    let result = ase.fetch_by_key(1).await;
    match result {
        Ok(ret) => {
          dbg!(ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("获取失败: {:?}", e);
          assert!(false);
        }
    }
}

#[tokio::test]
async fn get_by_field() {
    setup_db_pool().await;

    let ase = ArticleService::new();
    let qf = QueryCondition::from(|builder: &mut QueryBuilder| {
      builder.filter(field("a_class").eq("其他1"));
    });
    let result = ase.fetch_one(qf).await;
    match result {
        Ok(ret) => {
          dbg!(ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("获取失败: {:?}", e);
          assert!(false);
        }
    }
}

#[tokio::test]
async fn get_top() {
    setup_db_pool().await;
    
    let ase = ArticleService::new();
    let qf = QueryCondition::from(|builder: &mut QueryBuilder| {
      builder.order_by("a_id", false).limit_offset(5, None);
    });
    let result = ase.fetch_all(qf).await;

    match result {
        Ok(ret) => {
          dbg!(ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("查询失败: {:?}", e);
          assert!(false);
        }
    }
}
