# 🛠️ Kitx - A Fast CRUD Toolkit Based on Rust's Sqlx

🌐 English | [中文](https://github.com/wujianqi/kitx/blob/main/README.CN.md)

**A lightweight CRUD toolkit built on top of `sqlx::QueryBuilder`**

> Use it just like you'd use Sqlx — flexible, simple, and without extra overhead!  
> Supports: **SQLite**, **MySQL/MariaDB**, and **PostgreSQL**

---

## 🌟 Key Features

1. **Native Sqlx Usage Style**  
   Core queries are built with a thin wrapper around `sqlx::QueryBuilder`, ensuring type safety and protection against SQL injection. Easily compose raw SQL fragments for complex query scenarios.

2. **Simplified Entity Model Macros**  
   Depends only on the `FieldAccess` trait (besides `sqlx`). No heavy derive macros needed — minimal configuration and boilerplate. Comes with utility functions to parse entity models.

3. **Reduced Field Binding Effort**  
   Eliminates repetitive `.bind(x).bind(y)...` calls. Many operations require **no manual binding** of field values!

4. **Built-in Common Operations**  
   Provides ready-to-use methods for **Insert, Update, Upsert, Delete, Select**, including regular pagination and cursor-based pagination — covering most real-world use cases.

---

## 🚀 Why Choose Kitx? See It in Action!

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

## 📦 Getting Started

### 1. Add Dependency

```toml
[dependencies]
kitx = "0.0.15"
```

Or, if you're targeting a specific database (recommended for better compile-time performance):

```toml
# For PostgreSQL
kitx = { version = "0.0.15", features = ["postgres"] }

# For MySQL
kitx = { version = "0.0.15", features = ["mysql"] }

# For SQLite
kitx = { version = "0.0.15", features = ["sqlite"] }
```

> All three databases are supported by default, but enabling only the required feature improves compilation speed.

---

### 2. Usage Guide

```rust
use kitx::prelude::{*, postgres::*};

async fn test_find_all() {
    let qb = Select::<Article>::select_default().from_default().inner();
    
    init_pool().await;
    let list = fetch_all::<Article>(qb).await.unwrap();

    // ...
}
```

For more examples, check the integration tests under each database-specific module.

---

💡 **Note**:  
> Kitx works by breaking down SQL statements into keyword-based segments (e.g., `"SELECT {} FROM {} WHERE {}"`) and using entity model data to auto-fill placeholders. When automatic filling isn't sufficient, you can fall back to manual construction via closures (`fn(QueryBuilder)`), allowing aliases, joins, nested conditions, etc.  

> Methods named `one` or `many` that operate directly on entity models **do not support custom SQL fragments**. These require a strict naming convention: the database table name (snake_case) must correspond to the struct name (camelCase).  

> All methods are thoroughly unit-tested to ensure reliability.  

--- 

> ✅ Simple. Safe. Expressive.  
> Build powerful database interactions — without leaving the comfort of Sqlx.