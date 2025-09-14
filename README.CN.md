# 🛠️ Kitx - 基于rust Sqlx的快速增删改查工具包

🌐 中文 | [English](README.md) 

**基于 `sqlx::QueryBuilder` 封装的 CRUD 操作和工具包** 

> Sqlx怎么用，Kitx就怎么用，灵便简单，没有额外包袱！
> 支持sqlite、mysql/mariadb、postgresql

## 🌟 主要特点

1. **Sqlx原生使用方式**  
   主查询语句均基于 `sqlx::QueryBuilder`简单封装，保障类型安全，防止SQL注入攻击；也便于组合原生SQL片段，应对更复杂的查询场景；

2. **简化实体模型宏设置**  
   除 `sqlx` 外**仅依赖 `FieldAccess` trait**，无需复杂 derive 宏，减少大量配置，提供解析实体模型的工具函数包。

3. **减少字段项绑定**  
   减少大量`query.bind(x).bind(y)...` 的重复劳动，部分查询操作可以**无需手动绑定字段值**！

4. **内置常用操作方法**  
   提供 **Insert、Update、Upset、Delete、Select等多种CRUD方法**，包括普通分页、游标分页等，可覆盖大多数应用场景。


## 🚀 为什么选择它？看示例！

```rust
/// 查找数据列表，So easy ?
async fn test_find_all() {
   let qb = Select::<Article>::select_default().from_default().inner();
    
   init_pool().await;
   let list = fetch_all::<Article>(qb).await.unwrap();  
   dbg!(&list);
}

/// 不是ORM，但使用也很方便，弱点就是外键关联关系需手动处理
async fn test_update_one() {
   let mut entity = Article::new(110,"test_title_", None);
   entity.content = Some("test_content".to_string());
   entity.id = 1;

   let key = PrimaryKey::Single("id", true)
   let qb = Update::one(&entity, &key, true).unwrap();

   init_pool().await;
   let result = execute(qb).await.unwrap(); 
   println!("Updated {} rows.", result.rows_affected());
}

/// 嵌套子查询
async fn test_nested_subquery() {
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
      .order_by("id DESC")
      .inner();

   init_pool().await;
   let result = fetch_all::<Article>(qb).await.unwrap();
   dbg!(&result);
}
```

## 📦 快速开始

### 1. 添加依赖
```toml
[dependencies]
kitx = "0.0.15"

# For PostgreSQL
kitx = { version = "0.0.15", features = ["postgres"] }

# For MySQL
kitx = { version = "0.0.15", features = ["mysql"] }

# For SQLite
kitx = { version = "0.0.15", features = ["sqlite"] }
```
  默认三种数据库均可使用，但仅需要某一个数据库，请添加对应数据库的依赖，可优化编译性能。

### 2. 使用指南
```rust
use kitx::prelude::{*, postgres::*};

async fn test_find_all() {
   let qb = Select::<Article>::select_default().from_default().inner();
    
   init_pool().await;
   let list = fetch_all::<Article>(qb).await.unwrap();  

   //...
}
```
  更多使用例子，请参考各数据库类型下的builder的测试用例。


💡 **说明**: 
> Kitx本质是将语句按关键词分割、组成链式操作，如："SELECT {} FROM {} WHERE {}"，然后利用实体模型数据解构，自动填充{}，如无法满足条件就则使用手动填充，即fn(QueryBuilder)，手动填充可使用别名、关联查询、嵌套子句等；
> 部分直接操作实体模型的方法，名为many、one的方法，无法使用手动，且表名（蛇形命名）为与实体模型结构体名（驼峰命名）必须是对应关系。
> 每个方法都经过了单元侧试，确保功能正常。
