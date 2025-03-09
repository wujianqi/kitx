# KitX - Lightweight SQL Builder for Rust

A minimalistic SQL builder library based on [sqlx](https://crates.io/crates/sqlx), supporting SQLite, MySQL/MariaDB, and PostgreSQL. It offers efficient database operations with soft delete capabilities and global filters, enabling developers to interact with databases more effectively.

## Features

### Core Functionality
- **Efficient CRUD Operations**  
  `insert_one`, `insert_many`, `update_one`, `update_many`, `delete_one`, `delete_many` with transaction support

- **Advanced Queries**  
  `fetch_all`, `fetch_by_key`, `fetch_one`, `fetch_paginated`, `fetch_by_cursor`, `exists`, `count`

- **Soft Delete Management**  
  `restore_one`, `restore_many` with global configuration

- **Flexible Query Building**  
  Supports JOINs, CASE WHEN, aggregations, and custom SQL extensions

### Key Advantages
- 🚀 **No ORM Overhead** - Direct SQL interaction with builder pattern  
- 🔧 **Field Access API** - Utilizes [field_access](https://crates.io/crates/field_access) for field operations  
- 🌍 **Global Filters** - Apply tenant ID or soft delete filters across all queries  
- 📦 **Extensible** - Easily add custom operations and query modifiers  

## Quick Start

### 1. Add Dependency
```toml
# Default SQL Builder, completely decoupled from any external libraries.
kitx = "0.0.8"

# For SQLite only
kitx = { version = "0.0.8", features = ["sqlite"] }

# For MySQL/MariaDB only
kitx = { version = "0.0.8", features = ["mysql"] }

# For PostgreSQL only
kitx = { version = "0.0.8", features = ["postgres"] }
```

### 2. Basic Usage
```rust
use kitx::sqlite::{sql::QueryBuilder, sql::field, operations::Operations};

// SQL Builder Example
// AND and OR conditions can be applied either within filter clauses or directly in the builder.
let query = QueryBuilder::select("users", &["id", "name"])
    .filter(field("age").eq(23))
    .filter(field("salary").gt(4500))
    .or(field("status").in_vec(vec!["active", "pending"]))
    .order_by("created_at", false)
    .build_mut().0;

// CRUD Operations
let op = Operations::new("articles", ("article_id", true));
let article = Article {
    id: 42,
    title: "Rust Best Practices".into(),
    content: "...".into(),
};

// Insert with transaction
op.insert_one(article, true).await?;
```

### 3. Pagination Example
```rust
let results = op.fetch_paginated(10, 2, QueryCondition.empty()).await?;

let results = op.fetch_by_cursor(10, QueryCondition.from(..)).await?;

```

### 4. Optional: Global Configuration
```rust
// Soft delete configuration
set_global_soft_delete_field("deleted_at", vec!["audit_logs"]);

// Global_filter is applied on a per-thread basis.
// Multi-tenant filtering
set_global_filter(field("tenant_id").eq(123)), vec!["system_metrics"]);
```

## License
MIT License

