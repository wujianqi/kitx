use kitx::common::builder::BuilderTrait;
use kitx::sql::{builder::Builder, filter::Field, params::Value};

#[cfg(feature = "sqlite")]
use kitx::sqlite::sql::{field, QueryBuilder};

#[test]
fn sql_test() {
    let query = Builder::select("users", &["id", "name"])
        .filter(Field::<Value>::get("age").eq(23))
        .filter(Field::<Value>::get("salary").gt(45))
        .or(Field::<Value>::get("status").r#in(vec!["A", "B"]))
        .order_by("name", true)
        .order_by("age", false)
        .build_mut().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");
}

#[cfg(feature = "sqlite")]
#[test]
fn sql_sqlite_test() {
    let query = QueryBuilder::select("users", &["id", "name"])
        .filter(field("age").eq(23))
        .filter(field("salary").gt(45))
        .or(field("status").r#in(vec!["A", "B"]))
        .order_by("name", true)
        .order_by("age", false)
        .build_mut().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");
}
