## 基于Sqlx的数据库轻封装

暂只支持Sqlite、MySql。

**内置19个数据库操作方法**  

*增删改操作（基于事务）：*  
insert, insert_many, update, update_when, update_many, delete, delete_many, batch_delete, execute,  

*查询操作：*  
get_all, get_by_key, get_top, get_list_paginated, search, search_paginated，exists，is_unique，agg，join

---------------------

```rust
/// Sql builder 查询语句构建器
/// 示例
let query = Builder::select("users", &["id", "name"])
        .filter(field("age").eq(23))
        .and(field("salary").gt(45))
        .or(field("status").in_list(vec!["A", "B"]))
        .order_by("name", true)
        .order_by("age", false)
        .build().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");


/// 见test用例，修改数据示例
async fn update() {
    init_db_pool("sqlite:./my.db").await.unwrap();

    let article = Article {
      a_id: 20,
      a_class: Some("关于我们".to_string()),
      a_content: Some("测试文章内容".to_string()),
    };
    let result = ArticleService::as_ops(article)
      .update(true).await;

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
} 
```

--------------------

```toml
[dependencies]
kitx = "0.0.3"

```

---------------------
##### 数据库辅助管理（非本库）：

* sqlx cli 安装 : cargo install sqlx-cli  
* 创建/删除数据库: sqlx database create / sqlx database drop  
* 创建迁移脚本：sqlx migrate add <name>  
* 比较后运行  sqlx migrate run  
