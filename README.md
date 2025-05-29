# KitX: Lightweight Rust SQL Builder for Rapid CRUD Operations

A minimalistic SQL builder library for Rust built on [sqlx](https://crates.io/crates/sqlx), designed for streamlined database interactions. This lightweight wrapper focuses on accelerating core CRUD operations (Create, Read, Update, Delete) while maintaining simplicity for straightforward database management tasks.

## Features

### Core Functionality
- **Efficient CRUD Operations**  
  `insert_one`, `insert_many`, `update_by_key`, `update_by_expr`, `update_one`,  
  `upsert_by_key`, `upsert_many`, `delete_by_key`, `delete_by_cond`, `delete_many`  

- **Advanced Queries**  
  `get_one_by_key`, `get_one`, `get_list`, `get_list_paginated`,  
  `get_list_by_cursor`, `exists`, `count`  

- **Soft Delete Management**  
  `restore_one`, `restore_many`  
  with global configuration

- **Flexible Query Building**  
  Supports JOINs, CASE WHEN, WITH CTE, and aggregations. Supports ON CONFLICT/DUPLICATE KEY (upsert) and RETURNING for conflict resolution and data retrieval.

- **Code Characteristics** 
  - **No Macros**: Public interfaces avoid macros, ensuring transparency and maintainability.
  - **No `.unwrap()` or `.expect()`**: Prevents runtime panics by promoting robust error handling.


### Key Advantages
- 🚀 **No ORM Overhead** - Direct SQL interaction with builder pattern  
- 🌍 **Global Filters** - Apply tenant ID or soft delete filters across all queries  
- 📦 **Extensible** - Easily add custom operations and query modifiers  

## Quick Start

### 1. Add Dependency
```toml
# Default SQL Builder, completely decoupled from any external libraries.
kitx = "0.0.12"

# For SQLite only, WAL mode is enabled by default.
kitx = { version = "0.0.12", features = ["sqlite"] }

# For MySQL/MariaDB only
kitx = { version = "0.0.12", features = ["mysql"] }

# For PostgreSQL only
kitx = { version = "0.0.12", features = ["postgres"] }
```

### 2. Basic Usage
```rust
use kitx::prelude::{*, postgres::*};

// SQL Builder Example
// AND and OR conditions can be applied either within filter clauses or directly in the builder.
let query = Select::columns(&["id", "name"])
    .from("users")
    .and_where(Expr::col("age").eq(23))
    .and_where(Expr::col("salary").gt(4500))
    .or_where(Expr::col("status").is_in(vec!["active", "pending"]))
    .order_by("created_at", OrderBy::Desc)
    .build().0;

let query2 = Insert::into("users")
    .columns(&["id", "name"])
    .values(&[22, "John Doe"])
    .build().0;
  
// CRUD Operations (Single Key)
let op = Operations::new("articles", ("article_id", true));
// Composite Key Operations
// let op = MutliKeyOperations::new("articles_tag", vec!["article_id", "tag_id"]);

let article = Article {
    id: 22,
    title: "Rust Best Practices".into(),
    content: "...".into(),
};

// Insert with transaction
op.insert_one(article).await?;
```

### 3. Pagination Example
```rust
let results = op.get_list_paginated(10, 2, empty_query()).await?;

let results = op.get_list_by_cursor(10, |&mut builder|{
    builder.and_where_mut(Expr::col("created_at").gt(DateTime::now()));
}).await?;

```

### 4. Transaction Management
```rust
use kitx::prelude::{*, postgres::*};

let query = Query::shared();
let article_op = article_operations().set(query.share());
let article_tag_op = tag_operations().set(query.share());

query.share().begin_transaction().await?

let mut article = Article::new(100,"test222", None);
article.content = Some("abc".to_string());

let mut article_ag = ArticleTag::new("tag1");
article_ag.article_id = 1;
article_ag.share_seq = 1234;

article_op.insert_one(article).await?;
article_tag_op.insert_one(article_ag).await?;

query.share().commit().await?;

```

### 5. Optional: Global Configuration
```rust
// Soft delete configuration
set_global_soft_delete_field("deleted_at", &["audit_logs"]);

// Global_filter is applied on a per-thread basis.
// Multi-tenant filtering
set_global_filter(col("tenant_id").eq(123)), &["system_metrics"]);
```

### 6. More Usage Examples 
For more detailed usage examples and advanced scenarios, please refer to the test cases provided in the repository.

## License
MIT License