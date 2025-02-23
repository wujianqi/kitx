use crate::common::builder::BuilderCondition;
use crate::sql::{builder::Builder, filter::FieldValue};
use super::kind::DataKind;

/// Sqlite 专用的 SQL 构建器。
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

