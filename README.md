# 🛠️ Kitx - A Fast CRUD Toolkit Based on Rust's Sqlx

<div align="right">
  🌐 <a href="#readme">English</a> | <a href="#readme-中文">中文</a>
</div>

---

## <a id="readme"></a>English

**A lightweight CRUD toolkit built on top of `sqlx::QueryBuilder`**

> Use it just like you'd use Sqlx — flexible, simple, and without extra overhead!  
> Supports: **SQLite**, **MySQL/MariaDB**, and **PostgreSQL**

---

### 🌟 Key Features

1. **Native Sqlx Usage Style**  
   Core queries are built with a thin wrapper around `sqlx::QueryBuilder`, ensuring type safety and protection against SQL injection. Easily compose raw SQL fragments for complex query scenarios.

2. **Simplified Entity Model Macros**  
   Depends only on the `FieldAccess` trait (besides `sqlx`). No heavy derive macros needed — minimal configuration and boilerplate. Comes with utility functions to parse entity models.

3. **Reduced Field Binding Effort**  
   Eliminates repetitive `.bind(x).bind(y)...` calls. Many operations require **no manual binding** of field values!

4. **Built-in Common Operations**  
   Provides ready-to-use methods for **Insert, Update, Upsert, Delete, Select**, including regular pagination and cursor-based pagination — covering most real-world use cases.

---

### 🚀 Why Choose Kitx? See It in Action!

```rust
/// Fetch all records — So easy?
async fn test_find_all() {
    let qb = Select::<Article>::select_default().from_default().inner();
    
    init_pool().await;
    let list = fetch_all::<Article>(qb).await.unwrap();  
    dbg!(&list);
}
```

```rust
/// Not an ORM, but still very convenient.
/// Note: Foreign key relationships must be handled manually.
async fn test_update_one() {
    let mut entity = Article::new(110, "test_title_", None);
    entity.content = Some("test_content".to_string());
    entity.id = 1;

    let key = PrimaryKey::Single("id", true);
    let qb = Update::one(&entity, &key, true).unwrap();

    init_pool().await;
    let result = execute(qb).await.unwrap(); 
    println!("Updated {} rows.", result.rows_affected());
}
```

```rust
/// Nested subquery example
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
            b.push("views < ");
            avg_views_subquery.append_to(b);
        })
        .order_by("id DESC")
        .inner();

    init_pool().await;
    let result = fetch_all::<Article>(qb).await.unwrap();
    dbg!(&result);
}
```

---

### 📦 Getting Started

#### 1. Add Dependency

```toml
[dependencies]
kitx = "0.0.16"
```

Or, if targeting a specific database (recommended):

```toml
# For PostgreSQL
kitx = { version = "0.0.16", features = ["postgres"] }

# For MySQL
kitx = { version = "0.0.16", features = ["mysql"] }

# For SQLite
kitx = { version = "0.0.16", features = ["sqlite"] }
```

> All three databases are supported by default. Enabling only required features improves compile performance.

#### 2. Usage Guide

```rust
use kitx::prelude::{*, postgres::*};

async fn test_find_all() {
    let qb = Select::<Article>::select_default().from_default().inner();
    
    init_pool().await;
    let list = fetch_all::<Article>(qb).await.unwrap();

    // ...
}
```

For more examples, check integration tests under each database module.

---

💡 **Note**:  
> Kitx breaks down SQL statements into segments (e.g., `"SELECT {} FROM {} WHERE {}"`) and auto-fills placeholders using entity model data. When automatic filling isn't enough, use manual closures (`fn(QueryBuilder)`) for aliases, joins, or nested conditions.  

> Methods like `one` or `many` that operate directly on entities **do not support custom SQL fragments**. They require table name (snake_case) to match struct name (camelCase).  

> All methods are thoroughly tested for reliability.  

> ✅ Simple. Safe. Expressive.  
> Build powerful database interactions — without leaving the comfort of Sqlx.

---

<br>

<div align="center">

[Back to Top ⬆️](#readme) | [返回顶部 ⬆️](#readme-中文)

</div>

---

## <a id="readme-中文"></a>中文

**基于 `sqlx::QueryBuilder` 封装的 CRUD 操作和工具包**

> Sqlx怎么用，Kitx就怎么用，灵便简单，没有额外包袱！  
> 支持 **SQLite、MySQL/MariaDB、PostgreSQL**

---

### 🌟 主要特点

1. **Sqlx原生使用方式**  
   主查询语句均基于 `sqlx::QueryBuilder` 简单封装，保障类型安全，防止SQL注入；也便于组合原生SQL片段，应对更复杂的查询场景。

2. **简化实体模型宏设置**  
   除 `sqlx` 外仅依赖 `FieldAccess` trait，无需复杂 derive 宏，减少配置，提供解析实体模型的工具函数包。

3. **减少字段项绑定**  
   减少大量 `query.bind(x).bind(y)...` 的重复劳动，部分操作可以**无需手动绑定字段值**！

4. **内置常用操作方法**  
   提供 **Insert、Update、Upset、Delete、Select** 等多种 CRUD 方法，包括普通分页、游标分页等，可覆盖大多数应用场景。

---

### 🚀 为什么选择它？看示例！

```rust
/// 查找数据列表，So easy ?
async fn test_find_all() {
   let qb = Select::<Article>::select_default().from_default().inner();
    
   init_pool().await;
   let list = fetch_all::<Article>(qb).await.unwrap();  
   dbg!(&list);
}
```

```rust
/// 不是ORM，但使用也很方便，弱点就是外键关联关系需手动处理
async fn test_update_one() {
   let mut entity = Article::new(110,"test_title_", None);
   entity.content = Some("test_content".to_string());
   entity.id = 1;

   let key = PrimaryKey::Single("id", true);
   let qb = Update::one(&entity, &key, true).unwrap();

   init_pool().await;
   let result = execute(qb).await.unwrap(); 
   println!("Updated {} rows.", result.rows_affected());
}
```

---

### 📦 快速开始

#### 1. 添加依赖

```toml
[dependencies]
kitx = "0.0.16"

# For PostgreSQL
kitx = { version = "0.0.16", features = ["postgres"] }

# For MySQL
kitx = { version = "0.0.16", features = ["mysql"] }

# For SQLite
kitx = { version = "0.0.16", features = ["sqlite"] }
```

> 默认三种数据库均可使用，但仅需某一个时建议启用对应 feature，以优化编译性能。

#### 2. 使用指南

```rust
use kitx::prelude::{*, postgres::*};

async fn test_find_all() {
   let qb = Select::<Article>::select_default().from_default().inner();
    
   init_pool().await;
   let list = fetch_all::<Article>(qb).await.unwrap();  

   //...
}
```

更多使用例子，请参考各数据库类型下的 builder 测试用例。

---

💡 **说明**:  
> Kitx 本质是将语句按关键词分割、组成链式操作，如："SELECT {} FROM {} WHERE {}"，然后利用实体模型数据解构，自动填充{}。若无法满足条件，则使用手动填充 `fn(QueryBuilder)`，支持别名、关联查询、嵌套子句等。 

> 部分直接操作实体模型的方法（名为 `many`、`one` 的方法）无法使用手动填充，且表名（蛇形命名）必须与实体模型结构体名（驼峰命名）对应。  

> 每个方法都经过了单元测试，确保功能正常。

---

<div align="center">

[返回顶部 ⬆️](#readme-中文) | [Back to Top ⬆️](#readme)

</div>
