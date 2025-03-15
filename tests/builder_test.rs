use kitx::common::builder::BuilderTrait;
use kitx::sql::filter::{Field, FilterClause};
use kitx::sql::{builder::Builder, params::Value};
use kitx::sql::join::Join;
use kitx::sql::agg::Agg;
use kitx::sql::case_when::WhenClause;

#[cfg(feature = "sqlite")]
use kitx::sqlite::sql::{field, QueryBuilder};

#[test]
fn sql_test() {
    let query = Builder::select("users", &["id", "name"])
        .filter(Field::<Value>::get("age").eq(23))
        .filter(Field::get("salary").gt(45))
        .or(Field::get("status").r#in(vec!["A", "B"]))
        .order_by("name", true)
        .order_by("age", false)
        .build_mut().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");
}

#[test]
fn test_join() {
    // Test INNER JOIN with ON condition
    let sql = Builder::select("users", &["id", "name"])
        .filter(FilterClause::<Value>::new("users.age", "=", 25))
        .join(Join::inner("orders")
            .on(FilterClause::expr("users.id = orders.user_id")
            .and(FilterClause::expr("users.name = orders.user_name"))
    )).build_mut().0;

    assert_eq!(
        sql, 
        "SELECT id, name FROM users INNER JOIN orders ON users.id = orders.user_id AND users.name = orders.user_name WHERE users.age = ?"
    );
}

#[test]
fn test_aggregate_functions() {
    let sql = Builder::select("users", &[])
        .aggregate(Agg::<Value>::default()
            .count("id", Some("total_users"))
            .sum("age", Some("total_age"))
            .avg("age", Some("avg_age"))
            .min("age", Some("min_age"))
            .max("age", Some("max_age"))
            .group_by(&["department"])
            .having(FilterClause::new("COUNT(id)", ">", 10))
        ).build_mut().0;

    assert_eq!(
        sql,
        "SELECT * FROM users, COUNT(id) AS total_users, SUM(age) AS total_age, AVG(age) AS avg_age, MIN(age) AS min_age, MAX(age) AS max_age GROUP BY department HAVING COUNT(id) > ?"
    );
}

#[test]
fn test_case_when_builder() {
    let sql = Builder::new("SELECT id, name,", None)
        .case_when(WhenClause::<Value>::case()
            .when(FilterClause::new("age", ">", 18), "adult")
            .when(FilterClause::new("age", "<=", 18)
                .and(FilterClause::new("age", ">", 12)),"teenager")
            .else_result("child")
        ).append(" FROM users", None)
        .build_mut().0;

    let expected_sql = "SELECT id, name, CASE WHEN age > ? THEN adult WHEN age <= ? AND age > ? THEN teenager ELSE child END FROM users";
    assert_eq!(sql, expected_sql);
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
