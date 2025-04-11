# KitX - Lightweight SQL Builder for Rust

A minimalistic SQL builder library based on [sqlx](https://crates.io/crates/sqlx), supporting SQLite, MySQL/MariaDB, and PostgreSQL. It offers efficient database operations with soft delete capabilities and global filters, enabling developers to interact with databases more effectively.

## Features

### Core Functionality
- **Efficient CRUD Operations**  
  `insert_one`, `insert_many`, `update_by_key`, `update_one`, `upsert_by_key`,  
  `upsert_many`, `delete_by_key`, `delete_by_cond`, `delete_many`  
   with transaction support

- **Advanced Queries**  
  `get_list`, `get_by_key`, `get_one`, `get_list_paginated`, `get_list_by_cursor`,  
  `exists`, `count`

- **Soft Delete Management**  
  `restore_one`, `restore_many`  
  with global configuration

- **Flexible Query Building**  
  Supports JOINs, CASE WHEN, and aggregations. Provides native support for ON CONFLICT/DUPLICATE KEY (upsert) and RETURNING, enabling conflict resolution and data retrieval.

### Key Advantages
- 🚀 **No ORM Overhead** - Direct SQL interaction with builder pattern  
- 🔧 **Field Access API** - Utilizes [field_access](https://crates.io/crates/field_access) for field operations  
- 🌍 **Global Filters** - Apply tenant ID or soft delete filters across all queries  
- 📦 **Extensible** - Easily add custom operations and query modifiers  

## Quick Start

### 1. Add Dependency
```toml
# Default SQL Builder, completely decoupled from any external libraries.
kitx = "0.0.10"

# For SQLite only
kitx = { version = "0.0.10", features = ["sqlite"] }

# For MySQL/MariaDB only
kitx = { version = "0.0.10", features = ["mysql"] }

# For PostgreSQL only
kitx = { version = "0.0.10", features = ["postgres"] }
```

### 2. Basic Usage
```rust
use kitx::sqlite::{sql::Select, sql::Insert, sql::col, operations::Operations};

// SQL Builder Example
// AND and OR conditions can be applied either within filter clauses or directly in the builder.
let query = Select::columns(&["id", "name"])
    .from("users")
    .where_(col("age").eq(23))
    .and(col("salary").gt(4500))
    .or(col("status").in_(vec!["active", "pending"]))
    .order_by("created_at", false)
    .build().0;

let query2 = Insert::into("users")
    .columns(&["id", "name"])
    .values(&[22, "John Doe"])
    .build().0;

// CRUD Operations
// KitX does not support composite primary keys. For such cases, please use constraints instead.
let op = Operations::new("articles", ("article_id", true));
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

let results = op.get_list_by_cursor(10, Some(|&mut builder|{
    builder.where_mut(col("created_at").gt(DateTime::now()));
})).await?;

```

### 4. Optional: Global Configuration
```rust
// Soft delete configuration
set_global_soft_delete_field("deleted_at", vec!["audit_logs"]);

// Global_filter is applied on a per-thread basis.
// Multi-tenant filtering
set_global_filter(col("tenant_id").eq(123)), vec!["system_metrics"]);
```

## License
MIT License