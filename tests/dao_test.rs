<<<<<<< HEAD

use kitx::{
  //sqlite::connection::init_db_pool, 
  //sqlite::operations::DataOperations
  mysql::connection::init_db_pool, 
  mysql::operations::DataOperations
};

mod article;
use article::Article;
use article::ArticleService;

/* 
#[tokio::test]
async fn insert() {
    //init_db_pool("sqlite:./my.db").await.unwrap();
    init_db_pool("mysql://username:password@localhost/database_name").await.unwrap();
    
    let class = "关于我们";  
    let content= "真测试4444";
=======
use kitx::{
  sqlite::connection::init_db_pool, 
  sqlite::operations::DataOperations
};

mod article;
use crate::article::ArticleService;

/* #[tokio::test]
async fn insert() {
    init_db_pool("sqlite:./my.db").await.unwrap();
    
    let class = "关于我们";  
    let content= "真测试111";
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722

    let result = ArticleService::by_fields(class, content).insert().await;

    match result {
        Ok(id) => {
          println!("插入成功，ID: {}", id);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("插入失败: {:?}", e);
          assert!(false);
        }
    }
    //assert!(false);
} */

<<<<<<< HEAD

#[tokio::test]
async fn update() {
    //init_db_pool("sqlite:./my.db").await.unwrap();
    init_db_pool("mysql://username:password@localhost/database_name").await.unwrap();

    let article = Article {
      a_id: 2,
      a_class: Some("关于我们".to_string()),
      a_content: Some("测试修改内容".to_string()),
=======
/* 
#[tokio::test]
async fn update() {
    init_db_pool("sqlite:./my.db").await.unwrap();

    let article = Article {
      a_id: 20,
      a_class: Some("关于我们".to_string()),
      a_content: Some("真测试222".to_string()),
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
    };
    let result = ArticleService::as_ops(article).update(true).await;

    match result {
        Ok(num) => {
          println!("更新成功，数量: {}", num);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("更新失败: {:?}", e);
          assert!(false);
        }
    }
    //assert!(false);
<<<<<<< HEAD
}
=======
} */
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722

/* 
#[tokio::test]
async fn delete() {
    init_db_pool("sqlite:./my.db").await.unwrap();

    let result = ArticleService::by_key(26)
      .delete()
      .await;

    match result {
        Ok(num) => {
          println!("删除成功，数量: {}", num);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("删除失败: {:?}", e);
          assert!(false);
        }
    }
    //assert!(false);
} */

/* 
#[tokio::test]
async fn batch_delete() {
    init_db_pool("sqlite:./my.db").await.unwrap();

    let result = ArticleService::by_default()
      .delete_many(vec![26, 27, 28])
      .await;

    match result {
        Ok(num) => {
          println!("删除成功，数量: {}", num);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("删除失败: {:?}", e);
          assert!(false);
        }
    }
    //assert!(false);
} */

/* 
#[tokio::test]
async fn get_list() {
    init_db_pool("sqlite:./my.db").await.unwrap();

    let result = ArticleService::by_default()
      .get_list_paginated(2, 10)      
      .await;

    match result {
        Ok(entities) => {
          for entity in entities.items {
            println!("数据， {}", entity.a_class.unwrap_or_else(|| "null".to_string()));
          }
          //println!("数据列表， {}", entities.items.len());
          assert!(true);
        },
        Err(e) =>{
          eprintln!("获取失败: {:?}", e);
          assert!(false);
        }
    }
    //assert!(false);
} */

/* 
#[tokio::test]
async fn get_by_key() {
    init_db_pool("sqlite:./my.db").await.unwrap();

    let result = ArticleService::by_key(12).get_by_key().await;
    match result {
        Ok(entity) => {
          /* println!("获取单条数据: {}, {}",  
            entity.a_class.unwrap_or_else(|| "无分类".to_string()),
            entity.a_content.unwrap_or_else(|| "无内容".to_string())
          ); */
          dbg!(entity);
          assert!(false);
        },
        Err(e) =>{
          eprintln!("获取失败: {:?}", e);
          assert!(false);
        }
    }
    //assert!(false);
} */

<<<<<<< HEAD
/* #[tokio::test]
=======
#[tokio::test]
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
async fn get_top() {
    init_db_pool("sqlite:./my.db").await.unwrap();

    let result = ArticleService::by_default()
      .get_top(5).await;

    match result {
        Ok(entities) => {
          println!("查询成功，num: {}", entities.len());
          dbg!(entities);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("查询失败: {:?}", e);
          assert!(false);
        }
    }
    //assert!(false);
}
<<<<<<< HEAD
 */
=======
>>>>>>> b0e16200fd5be220e3fbdfdf161f92205821e722
