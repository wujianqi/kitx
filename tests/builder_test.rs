use std::borrow::Cow;

use kitx::common::builder::{BuilderTrait, FilterTrait};
use kitx::common::types::OrderBy;
use kitx::sql::query_builder::SqlBuilder;
use kitx::sql::cte::{WithCTE, CTE};
use kitx::sql::delete::DeleteBuilder;
use kitx::sql::insert::InsertBuilder;
use kitx::sql::bind_params::Value;
use kitx::sql::update::UpdateBuilder;

use kitx::sql::filter::Expr;

use kitx::sql::select::SelectBuilder;
use kitx::sql::join::JoinType;
use kitx::sql::agg::Func;
use kitx::sql::case_when::CaseWhen;

#[test]
fn builder_test() {
    let sql = SqlBuilder::<Value>::raw(
        r#"SELECT id, name FROM users WHERE age = ? AND id = ?"#, 
        Some(vec![23, 22]))
    .build().0;
    assert_eq!(sql, "SELECT id, name FROM users WHERE age = ? AND id = ?")
}

#[test]
fn select_test() {
    let query = SelectBuilder::columns(&["id", "name"])
        .from("users")
        .and_where(Expr::<Value>::col("age").eq(23))
        .and_where(Expr::col("salary").gt(45))
        .or_where(Expr::col("status").is_in(vec!["A", "B"]))
        .order_by("name", OrderBy::Asc)
        .order_by("age", OrderBy::Desc)
        .build().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");
}

#[test]
fn select_with_limit_offset_test() {
    let builder = SelectBuilder::<Value>::columns(&["id", "name"])
        .from("users")
        .limit_offset(10, Some(5));

    //let query = builder.build().0;
    //assert_eq!(query, "SELECT id, name FROM users LIMIT ? OFFSET ?");

    let mut bd = builder;
        bd.and_where_mut(Expr::col("salary").gt(25));

    assert_eq!(bd.build().0, "SELECT id, name FROM users WHERE salary > ? LIMIT ? OFFSET ?");
}

#[test]
fn insert_test() {
    let query = InsertBuilder::into("users")
        .columns(&["name", "age"])
        .values(vec![
            vec![
                Value::Text(Cow::Borrowed("John")),
                Value::Int(30),
            ],
            vec![
                Value::Text(Cow::Borrowed("Jane")),
                Value::Int(25),
            ],
        ])
        .build().0;

    assert_eq!(query, "INSERT INTO users (name, age) VALUES (?, ?), (?, ?)");
}

#[test]
fn update_test() {
    let query = UpdateBuilder::<Value>::table("users")
        .set_cols(&["name", "age"], vec![
            Value::Text(Cow::Borrowed("John")),
            Value::Int(30),
        ],)
        .and_where(Expr::col("age").eq(23))
        .and_where(Expr::col("salary").gt(45))
        .or_where(Expr::col("status").is_in(vec!["A", "B"]))
        .build().0;

    assert_eq!(query, "UPDATE users SET name = ?, age = ? WHERE age = ? AND salary > ? OR status IN (?, ?)");
}
#[test]
fn delete_test() {
    let query = DeleteBuilder::<Value>::from("users")
        .and_where(Expr::<Value>::col("age").eq(23))
        .and_where(Expr::col("salary").gt(45))
        .or_where(Expr::col("status").is_in(vec!["A", "B"]))
        .build().0;

    assert_eq!(query, "DELETE FROM users WHERE age = ? AND salary > ? OR status IN (?, ?)");    
}

#[test]
fn test_join() {
    // Test INNER JOIN with ON condition
    let sql = SelectBuilder::columns(&["id", "name"])
        .from("users")
        .and_where(Expr::<Value>::new("users.age", "=", 25))
        .join(JoinType::inner("orders")
            .on(Expr::from_str("users.id = orders.user_id")
            .and(Expr::from_str("users.name = orders.user_name"))
    )).build().0;

    assert_eq!(
        sql, 
        "SELECT id, name FROM users INNER JOIN orders ON users.id = orders.user_id AND users.name = orders.user_name WHERE users.age = ?"
    );
}

#[test]
fn test_aggregate_functions() {
    let sql = SelectBuilder::columns(&["department"])
        .aggregate(Func::<Value>::default()
            .count("id", "total_users")
            .sum("age", "total_age")
            .avg("age", "avg_age")
            .min("age", "min_age")
            .max("age", "max_age")
            .group_by(&["department"])
            .having(Expr::col("COUNT(id)").gt(10))
        )
        .from("users")
        .and_where(Expr::col("age").lt(18))
        .build().0;

    assert_eq!(
        sql,
        "SELECT department, COUNT(id) AS total_users, SUM(age) AS total_age, AVG(age) AS avg_age, MIN(age) AS min_age, MAX(age) AS max_age FROM users WHERE age < ? GROUP BY department HAVING COUNT(id) > ?"
    );
}

#[test]
fn test_update_with_cte() {
    let subquery = SelectBuilder::columns(&["id", "name"])
        .from("users")
        .and_where(Expr::new("age", ">", 18));

    let cte = CTE::new("adult_users", subquery);

    let mut with_cte = WithCTE::new();
    with_cte.add_cte(cte);

    let update_query = UpdateBuilder::table("employees")
        .set("salary", 10000)
        .and_where(
            Expr::in_subquery("id", SelectBuilder::columns(&["id"]).from("adult_users")
        ))
        .with(with_cte);

    let (sql, _) = update_query.build();

    let expected_sql = r#"WITH adult_users AS (SELECT id, name FROM users WHERE age > ?) UPDATE employees SET salary = ? WHERE id IN (SELECT id FROM adult_users)"#;
    assert_eq!(sql.trim(), expected_sql.trim());

}

#[test]
fn test_case_when_builder() {
    let sql = SelectBuilder::columns(&["id, name"])
        .case_when(CaseWhen::<Value>::case()
            .when(Expr::col("age").gt(18), "adult")
            .when(Expr::col("age").lte(18)
                .and(Expr::col("age").gt(12)),"teenager")
            .otherwise("child")
        )
        .from("users")
        .build().0;

    let expected_sql = "SELECT id, name, CASE WHEN age > ? THEN adult WHEN age <= ? AND age > ? THEN teenager ELSE child END FROM users";
    assert_eq!(sql, expected_sql);
}

#[cfg(feature = "sqlite")]
#[test]
fn sql_sqlite_test() {
    use kitx::sqlite::Select;

    let query = Select::columns(&["id", "name "])
        .from("users")        
        .and_where(Expr::col("age").eq(23))
        .and_where(Expr::col("salary").gt(45))
        .or_where(Expr::col("status").is_in(vec!["A", "B"]))
        .order_by("name", OrderBy::Asc)
        .order_by("age", OrderBy::Desc)
        .build().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");
}

#[cfg(feature = "sqlite")]
#[test]
fn upsert_test() {
    use kitx::sqlite::{kind::DataKind, Insert};

    let query = Insert::into("users")
        .columns(&["id", "name", "age"])
        .values(vec![
            vec![
                DataKind::from(1),
                DataKind::from("Alice"),
                DataKind::from(25),
            ],
        ])
        .on_conflict_do_update("id", &["name", "age",])
        .build().0;

    let expected_sql = "INSERT INTO users (id, name, age) VALUES (?, ?, ?) \
                        ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, age = EXCLUDED.age";

    assert_eq!(query, expected_sql);
}