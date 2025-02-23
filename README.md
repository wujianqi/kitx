## 基于Sqlx的数据库轻封装

暂只支持Sqlite、MySql。

**内置15个数据库操作方法**  

*增删改操作（基于事务）：*  
insert_one, insert_many, update_one, update_many, delete_one, delete_many,

*查询操作：*  
fetch_all, fetch_by_key, fetch_one, fetch_paginated, fetch_by_cursor, exist, count, 

*软删除恢复操作：*  
restore_one, restore_many  

---------------------

```rust
/// Sql builder 查询语句构建器
/// 示例
fn sql_test() {
    let query = QueryBuilder::select("users", &["id", "name"])
        .filter(field("age").eq(23))
        .filter(field("salary").gt(45))
        .or(field("status").in_list(vec!["A", "B"]))
        .order_by("name", true)
        .order_by("age", false)
        .build_mut().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");
}


/// 见test用例，修改数据示例
async fn update() {
    setup_db_pool().await;

    let article = Article {
      a_id: 2,
      a_class: Some("关于我们".to_string()),
      a_content: Some("测试修改内容".to_string()),
    };
    let ase = ArticleService::new();
    let result = ase.update_one(article, false).await;

    match result {
        Ok(ret) => {
          println!("更新成功: {:?}", ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("更新失败: {:?}", e);
          assert!(false);
        }
    }
}
```

--------------------

```toml
[dependencies]
kitx = "0.0.4"

```

---------------------
##### 数据库辅助管理（非本库）：

* sqlx cli 安装 : cargo install sqlx-cli  
* 创建/删除数据库: sqlx database create / sqlx database drop  
* 创建迁移脚本：sqlx migrate add <name>  
* 比较后运行  sqlx migrate run  
