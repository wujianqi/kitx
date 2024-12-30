use kitx::sqlite::sql::{SQLBuilder, field};

#[test]
fn sql_test() {
    let query = SQLBuilder::select("users", &["id", "name"])
        .filter(field("age").eq(23))
        .and(field("salary").gt(45))
        .or(field("status").in_list(vec!["A", "B"]))
        .order_by("name", true)
        .order_by("age", false)
        .build().0;

    assert_eq!(query, "SELECT id, name FROM users WHERE age = ? AND salary > ? OR status IN (?, ?) ORDER BY name ASC, age DESC");
}