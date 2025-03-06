## Lightweight Database Wrapper Based on Sqlx

Currently supports only SQLite and MySQL (MariaDB).

### Features

**Built-in Database Operations**

- **CRUD Operations (Transaction-based):**
  - `insert_one`, `insert_many`, `update_one`, `update_many`, `delete_one`, `delete_many`

- **Query Operations:**
  - `fetch_all`, `fetch_by_key`, `fetch_one`, `fetch_paginated`, `fetch_by_cursor`, `exist`, `count`

- **Soft Delete and Restore Operations:**
  - `restore_one`, `restore_many`

**Custom extensions are supported.**
- The SQL builder supports JOIN, CASE WHEN, aggregation queries, and other operations. Additionally, you can extend the Operations methods..

### Notes

- This library is not an ORM library. It is an SQL statement builder based on [sqlx](https://crates.io/crates/sqlx).
- Instead of using custom macros for entity structs, we use the [field_access](https://crates.io/crates/field_access) crate, which makes it easier to operate on entity properties. This approach ensures simplicity and reduces coupling.


#### SQL Builder Example

```rust

fn sql_test() {
    let query = QueryBuilder::select("users", &["id", "name"])
        .filter(field("age").eq(23))
        .filter(field("salary").gt(45))
        .or(field("status").r#in(vec!["A", "B"]))
        .order_by("name", true)
        .order_by("age", false)
        .build_mut().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");
}
```

#### Data Modification Example  

```rust
async fn update() {
    setup_db_pool().await;

    let article = Article {
      a_id: 2,
      a_class: Some("about".to_string()),
      a_content: Some("content".to_string()),
    };
    let op:Operations<'static, Article> = Operations::new("article", ("a_id", true));
    let result = op.update_one(article, false).await;

    match result {
        Ok(ret) => {
          println!("{:?}", ret);
          assert!(true);
        },
        Err(e) =>{
          eprintln!("{:?}", e);
          assert!(false);
        }
    }
}
```

### Configuration Examples
```rust
/// field_name: The name of the field used for soft deletes.
/// exclude_tables: A list of table names to exclude from this behavior.
set_global_soft_delete_field("is_deleted", vec!["users"]);

/// filter: A tuple containing the filter clause and a list of tables to exclude from this filter.
set_global_filter((field("tenant_id").eq(20), vec!["logs"]));

/// The above configuration is not mandatory. No other changes are required.

```

--------------------

```toml
[dependencies]
kitx = "0.0.5"  # default sqlite

# Uncomment the following lines to use MySQL instead of SQLite
# kitx = { version = "0.0.5", features = ["mysql"] } 

```

#### Rust Version
It is recommended to use Rust version 1.85.0.
