use crate::common::builder::{BuilderCondition, BuilderTrait};
use crate::sql::{builder::Builder, filter::FieldValue};
use super::kind::DataKind;

/// MySQL 专用的 SQL 构建器。
pub type QueryBuilder<'a> = Builder<DataKind<'a>>;
pub type QueryCondition<'a> = BuilderCondition<'a, QueryBuilder<'a>>;

/// 创建一个用于获取字段值的对象。
///
/// # 参数
/// - `name`: 字段名。
///
/// # 返回
/// - `FieldValue`: 用于获取字段值的对象。
pub fn field<'a>(name: &'a str) -> FieldValue<'a, DataKind<'a>> {
    FieldValue::get(name)
}

// MySQL 特定方法
impl<'a> QueryBuilder<'a> {
    /// 添加 ON DUPLICATE KEY UPDATE 子句。
    pub fn on_duplicate_key_update(
        &mut self,
        table: &str,
        columns: &[&str],
        values: Vec<Vec<DataKind<'a>>>,
        update_columns: &[&str],
    ) -> &mut Self { // 返回 &mut Self
        // 复用 insert_into 的逻辑生成基础 SQL 和参数
        let mut builder = Builder::insert_into(table, columns, values.clone());

        // 添加 ON DUPLICATE KEY UPDATE 子句
        let update_clause = update_columns
            .iter()
            .map(|col| format!("{} = ?", col))
            .collect::<Vec<String>>()
            .join(", ");
        let sqlstr = format!(" ON DUPLICATE KEY UPDATE {}", update_clause);

        // 将需要更新的值再次添加到 cols_values 中
        let mut vals = Vec::new();
        for row in &values {
            for col in update_columns {
                if let Some(index) = columns.iter().position(|&c| c == *col) {
                    vals.push(row[index].clone());
                }
            }
        }

        // 将 SQL 和参数添加到 builder 中
        builder.append(&sqlstr, Some(vals));

        // 返回当前构建器的可变引用
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let values = vec![
            vec![DataKind::String("John".into()), DataKind::String("30".into())]
        ];
        let builder = QueryBuilder::insert_into("users", &["name", "age"], values);
        assert_eq!(builder.build().0, "INSERT INTO users ( name, age ) VALUES (?, ?)");
    }
}