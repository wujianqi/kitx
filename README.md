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
    let qb = Select::<Article>::table().finish();

    let pool = connection::get_db_pool().unwrap();
    let result = qb.build_query_as::<Article>().fetch_all(&*pool).await.unwrap(); 
    println!("{}", result.len();
}
```

```rust
/// Not an ORM, but still very convenient.
/// Note: Foreign key relationships must be handled manually.
async fn test_update_one() {
    let mut entity = Article::new(110, "test_title_", None);
    entity.content = Some("test_content".to_string());
    entity.id = 1;

    const KEY = PrimaryKey::Single("id", true);
    let qb = Update::one(&entity, &KEY, true).unwrap();

    let pool = connection::get_db_pool().unwrap();
    let result = qb.build().execute(&*pool).await.unwrap();
    println!("Updated {} rows.", result.rows_affected());
}
```

```rust
/// Find list paginated.
async fn test_find_list_paginated() {
   let page_number = 1;
   let page_size = 10;

   let qb = Select::<Article>::table()
      .order_by("id", Order::Desc)
      .paginate(page_number, page_size).unwrap();

   let pool = connection::get_db_pool().unwrap();
   let list = qb.build_query_as::<Article>().fetch_all(&*pool).await.unwrap();

   let qb2 = Select::<Article>::table()
      .columns(|b| {
         b.push("count(*)");
      })
      .finish();
   
   let total = qb2.build_query_scalar::<u64>().fetch_one(&*pool).await.unwrap();

   let pr = PaginatedResult::new(list, total, page_number, page_size);
   // ...
}

/// Using a CTE with subqueries in the same SQL query.
async fn test_with_cte() {
   let mut cte_builder = QueryBuilder::new("WITH article_cte AS ");
   Subquery::<Article>::table()            
      .filter( |b| {
            b.push("id > ").push_bind(50.into());
      })
      .append_to(&mut cte_builder);

   let qb = Select::<Article>::from_query_with_table(cte_builder, "article_cte")
      .finish();   

   let pool = connection::get_db_pool().unwrap();
   let list = qb.build_query_as::<Article>().fetch_all(&*pool).await.unwrap();

   // ...
}

```

---

### 📦 Getting Started

#### 1. Add Dependency

```toml
[dependencies]
kitx = "0.0.18"
```

Or, if targeting a specific database (recommended):

```toml
# For PostgreSQL
kitx = { version = "0.0.18", features = ["postgres"] }

# For MySQL
kitx = { version = "0.0.18", features = ["mysql"] }

# For SQLite
kitx = { version = "0.0.18", features = ["sqlite"] }
```

> All three databases are supported by default. Enabling only required features improves compile performance.

#### 2. Usage Guide

```rust
use kitx::prelude::{*, postgres::*};

async fn test_find_all() {
    let qb = Update::<Article>::table()
      .custom(|b| {
            b.push("views = views + 1");
        })
      .finish();

    // ...
}
```

For more examples, check integration tests under each database module.

---

## 1. Insert Builder

| Method | Description | Example |
|--------|-------------|---------|
| `one` | Creates a single record insert operation | `Insert::one(&entity, &PRIMARY_KEY).unwrap()` |
| `many` | Creates multiple records insert operation | `Insert::many(&entities, &PRIMARY_KEY).unwrap()` |
| `table` | Creates an insert operation with the default table name | `Insert::<Article>::table()` |
| `with_table` | Creates an insert operation with a custom table name | `Insert::with_table("custom_table", ...)` |
| `from_query` | Creates an Insert instance from a query | `Insert::from_query(query_builder, ...)` |
| `from_query_with_table` | Creates an Insert instance from a query with a custom table name | `Insert::from_query_with_table(query_builder, "custom_table", ...)` |
| `custom` | Customizes VALUES or value-related query statements | `Insert::table().custom(|b| b.push("..."))` |
| `returning` | Adds RETURNING clause to the insert statement (**PostgreSQL and SQLite only**) | `Insert::table().custom(...).returning("id")` |
| `finish` | Completes building and returns the internal QueryBuilder | `Insert::table().custom(...).finish()` |

## 2. Update Builder

| Method | Description | Example |
|--------|-------------|---------|
| `one` | Creates a single entity update operation | `Update::one(&entity, &PRIMARY_KEY, true).unwrap()` |
| `table` | Creates an Update instance with the default table name | `Update::<Article>::table()` |
| `with_table` | Creates an Update instance with a custom table name | `Update::with_table("custom_table", ...)` |
| `from_query` | Creates an Update instance from a query | `Update::from_query(query_builder, ...)` |
| `from_query_with_table` | Creates an Update instance from a query with a custom table name | `Update::from_query_with_table(query_builder, "custom_table", ...)` |
| `custom` | Customizes SET columns or other query statements | `Update::table().custom(|b| b.push("views = views + 1"))` |
| `filter` | Adds WHERE condition to the update statement | `Update::table().filter(|b| b.push("id = ").push_bind(1))` |
| `returning` | Adds RETURNING clause to the update statement (**PostgreSQL and SQLite only**) | `Update::table().custom(...).returning("id")` |
| `finish` | Completes building and returns the internal QueryBuilder | `Update::table().custom(...).finish()` |

## 3. Upsert Builder

| Method | Description | Example |
|--------|-------------|---------|
| `one` | Creates a single record upsert operation | `Upsert::one(&entity, &PRIMARY_KEY).unwrap()` |
| `many` | Creates multiple records upsert operation | `Upsert::many(&entities, &PRIMARY_KEY).unwrap()` |

## 4. Delete Builder

| Method | Description | Example |
|--------|-------------|---------|
| `table` | Creates a Delete instance with the default table name | `Delete::<Article>::table()` |
| `with_table` | Creates a Delete instance with a custom table name | `Delete::with_table("custom_table", ...)` |
| `from_query` | Creates a Delete instance from a query | `Delete::from_query(query_builder, ...)` |
| `from_query_with_table` | Creates a Delete instance from a query with a custom table name | `Delete::from_query_with_table(query_builder, "custom_table", ...)` |
| `by_primary_key` | Creates a DELETE query by primary key | `Delete::table().by_primary_key(&PRIMARY_KEY, &ids)` |
| `filter` | Creates a DELETE query with custom WHERE conditions | `Delete::table().filter(|b| b.push("id = ").push_bind(1))` |
| `returning` | Adds RETURNING clause to the DELETE statement (**PostgreSQL and SQLite only**) | `Delete::table().returning("*")` |
| `finish` | Completes building and returns the internal QueryBuilder | `Delete::table().finish()` |

## 5. Select Builder

| Method | Description | Example |
|--------|-------------|---------|
| `table` | Creates a Select instance with the default table name | `Select::<Article>::table()` |
| `with_table` | Creates a Select instance with a custom table name | `Select::with_table("custom_table", ...)` |
| `from_query` | Creates a Select instance from a query | `Select::from_query(query_builder, ...)` |
| `from_query_with_table` | Creates a Select instance from a query with a custom table name | `Select::from_query_with_table(query_builder, "custom_table", ...)` |
| `columns` | Creates a custom column query statement | `Select::table().columns(|b| b.push("id, title"))` |
| `filter` | Creates a SELECT query with custom WHERE conditions | `Select::table().filter(|b| b.push("id > ").push_bind(10))` |
| `join` | Creates a JOIN query statement | `Select::table().join("JOIN comments ON ...")` |
| `group_by` | Creates a GROUP BY query statement | `Select::table().group_by("category_id")` |
| `having` | Creates a HAVING clause | `Select::table().having(|b| b.push("COUNT(*) > 1"))` |
| `by_primary_key` | Creates a SELECT query by primary key | `Select::table().by_primary_key(&PRIMARY_KEY, &ids)` |
| `order_by` | Creates an ORDER BY clause | `Select::table().order_by("id", Order::Desc)` |
| `paginate` | Creates a pagination query statement | `Select::table().paginate(1, 10).unwrap()` |
| `cursor` | Creates a cursor pagination query statement | `Select::table().cursor("id", Order::Asc, None, 10).unwrap()` |
| `finish` | Completes building and returns the internal QueryBuilder | `Select::table().finish()` |

## 6. Subquery Builder

| Method | Description | Example |
|--------|-------------|---------|
| `table` | Creates a subquery with the default table name | `Subquery::<Article>::table()` |
| `with_table` | Creates a subquery with a custom table name | `Subquery::with_table("custom_table")` |
| `columns` | Adds custom columns to the subquery | `Subquery::table().columns(|b| b.push("AVG(views)"))` |
| `filter` | Adds WHERE condition to the subquery | `Subquery::table().filter(|b| b.push("id > ").push_bind(3))` |
| `join` | Adds JOIN clause to the subquery | `Subquery::table().join("JOIN comments ON ...")` |
| `group_by` | Adds GROUP BY clause to the subquery | `Subquery::table().group_by("category_id")` |
| `having` | Adds HAVING clause to the subquery | `Subquery::table().having(|b| b.push("COUNT(*) > 1"))` |
| `append_to` | Embeds the subquery into a parent query builder | `subquery.append_to(&mut parent_query)` |

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

### 📦 使用指南

更多使用例子，请参考文档、各数据库类型下的 builder 测试用例。

---

💡 **说明**:  
> Kitx 本质是将语句按关键词分割、组成链式操作，如："SELECT {} FROM {} WHERE {}"，然后利用实体模型数据解构，自动填充{}。若无法满足条件，则使用手动填充 `fn(QueryBuilder)`，支持别名、关联查询、嵌套子句等。 

> 部分直接操作实体模型的方法（名为 `one`、`many` 的方法）无法使用手动填充，且表名（蛇形命名）必须与实体模型结构体名（驼峰命名）对应。  

> 每个方法都经过了单元测试，确保功能正常。

---

<div align="center">

[返回顶部 ⬆️](#readme-中文) | [Back to Top ⬆️](#readme)

</div>
