//! Field extraction utilities for database operations.
//! 
//! This module provides comprehensive utilities for extracting field names and values
//! from structs that implement the `FieldAccess` trait. It supports various extraction
//! modes including filtering, batch processing, and conditional value extraction.
//! These utilities are essential for building dynamic SQL queries and parameter binding.
//! 
//! 数据库操作的字段提取工具。
//! 
//! 此模块为实现了 `FieldAccess` trait 的结构体提供全面的字段名和值提取工具。
//! 支持多种提取模式，包括过滤、批处理和条件值提取。
//! 这些工具对于构建动态SQL查询和参数绑定至关重要。

use field_access::{FieldAccess, Fields};

use super::conversion::{ValueConvert, is_empty_or_none};

/// Extract all fields and values from a struct.
/// 
/// This function extracts all field names and their corresponding values from
/// a struct's fields iterator. It's useful when you need to process all fields
/// without any filtering or conditions.
/// 
/// # Type Parameters
/// * `VAL` - The target value type that implements `ValueConvert`
/// 
/// # Arguments
/// * `fields` - An iterator over field names and their access objects
/// 
/// # Returns
/// A tuple containing:
/// * `Vec<&str>` - Vector of field names
/// * `Vec<VAL>` - Vector of converted field values
/// 
/// # Examples
/// ```rust
/// use kitx::common::fields::extract_all;
/// use sqlx::types::Json;
/// 
/// // Assuming entity implements FieldAccess
/// let (names, values): (Vec<&str>, Vec<String>) = extract_all(entity.fields());
/// assert_eq!(names.len(), values.len());
/// ```
/// 
/// 从结构体中提取所有字段名和值。
/// 
/// 此函数从结构体的字段迭代器中提取所有字段名及其对应的值。
/// 当需要处理所有字段而不进行任何过滤或条件判断时很有用。
/// 
/// # 类型参数
/// * `VAL` - 实现了 `ValueConvert` 的目标值类型
/// 
/// # 参数
/// * `fields` - 字段名及其访问对象的迭代器
/// 
/// # 返回值
/// 包含以下内容的元组：
/// * `Vec<&str>` - 字段名向量
/// * `Vec<VAL>` - 转换后的字段值向量
/// 
/// # 示例
/// ```rust
/// use kitx::common::fields::extract_all;
/// use sqlx::types::Json;
/// 
/// // 假设 entity 实现了 FieldAccess
/// let (names, values): (Vec<&str>, Vec<String>) = extract_all(entity.fields());
/// assert_eq!(names.len(), values.len());
/// ```
pub fn extract_all<VAL>(fields: Fields) -> (Vec<&str>, Vec<VAL>) 
where 
    VAL: ValueConvert,
{
    let mut cols_names = Vec::with_capacity(fields.len());
    let mut cols_values = Vec::with_capacity(fields.len());

    for (name, field) in fields {
        cols_names.push(name);
        cols_values.push(VAL::convert(field.as_any()));
    }
    (cols_names, cols_values)
}

/// Extract fields from a struct with filtering and a custom bind function.
/// 
/// This function provides advanced field extraction with multiple filtering options
/// and allows for custom processing of each field through a bind function. It's
/// particularly useful for SQL parameter binding where you need to process each
/// field value as it's extracted.
/// 
/// # Type Parameters
/// * `VAL` - The target value type that implements `ValueConvert`
/// * `F` - A closure type that processes each field name and value
/// 
/// # Arguments
/// * `fields` - An iterator over field names and their access objects
/// * `filter_columns` - Slice of column names to exclude from extraction
/// * `skip_non_null` - If true, skip fields that are empty or None
/// * `bind_fn` - A closure called for each extracted field with (name, value)
/// 
/// # Returns
/// A tuple containing:
/// * `Vec<&'static str>` - Vector of extracted field names
/// * `Vec<VAL>` - Vector of converted field values
/// 
/// # Examples
/// ```rust
/// use kitx::common::fields::extract_with_bind;
/// 
/// let mut params = Vec::new();
/// let (names, values) = extract_with_bind(
///     entity.fields(),
///     &["id"], // Skip 'id' field
///     true,    // Skip null values
///     |name, value: String| {
///         params.push(format!("{}={}", name, value));
///     }
/// );
/// ```
/// 
/// 从结构体中提取字段名和值，支持过滤和自定义绑定函数。
/// 
/// 此函数提供高级字段提取功能，具有多种过滤选项，并允许通过绑定函数
/// 对每个字段进行自定义处理。特别适用于SQL参数绑定，
/// 需要在提取每个字段值时进行处理的场景。
/// 
/// # 类型参数
/// * `VAL` - 实现了 `ValueConvert` 的目标值类型
/// * `F` - 处理每个字段名和值的闭包类型
/// 
/// # 参数
/// * `fields` - 字段名及其访问对象的迭代器
/// * `filter_columns` - 要从提取中排除的列名切片
/// * `skip_non_null` - 如果为true，跳过空值或None的字段
/// * `bind_fn` - 为每个提取的字段调用的闭包，参数为(name, value)
/// 
/// # 返回值
/// 包含以下内容的元组：
/// * `Vec<&'static str>` - 提取的字段名向量
/// * `Vec<VAL>` - 转换后的字段值向量
/// 
/// # 示例
/// ```rust
/// use kitx::common::fields::extract_with_bind;
/// 
/// let mut params = Vec::new();
/// let (names, values) = extract_with_bind(
///     entity.fields(),
///     &["id"], // 跳过 'id' 字段
///     true,    // 跳过空值
///     |name, value: String| {
///         params.push(format!("{}={}", name, value));
///     }
/// );
/// ```
pub fn extract_with_bind<VAL, F>(
    fields: Fields,
    filter_columns: &[&str],
    skip_non_null: bool,
    mut bind_fn: F
) -> (Vec<&'static str>, Vec<VAL>)
where
    VAL: ValueConvert,
    F: FnMut(&str, VAL)
{
    let mut cols_names = Vec::new();
    let mut cols_values = Vec::new();

    for (name, field) in fields {
        if filter_columns.contains(&name) {
            continue;
        }

        let any_value = field.as_any();        
        if skip_non_null && is_empty_or_none(any_value) {
            continue;
        }
        cols_names.push(name);
        cols_values.push(VAL::convert(any_value));        
        bind_fn(name, VAL::convert(any_value));
    }
    (cols_names, cols_values)
}

/// Extract fields from a struct with filtering options.
/// 
/// This is a simplified version of `extract_with_bind` that provides field
/// extraction with filtering capabilities but without custom processing.
/// It's ideal for basic field extraction scenarios where you need to exclude
/// certain columns or skip null values.
/// 
/// # Type Parameters
/// * `VAL` - The target value type that implements `ValueConvert`
/// 
/// # Arguments
/// * `fields` - An iterator over field names and their access objects
/// * `filter_columns` - Slice of column names to exclude from extraction
/// * `skip_non_null` - If true, skip fields that are empty or None
/// 
/// # Returns
/// A tuple containing:
/// * `Vec<&'static str>` - Vector of extracted field names
/// * `Vec<VAL>` - Vector of converted field values
/// 
/// # Examples
/// ```rust
/// use kitx::common::fields::extract_with_filter;
/// 
/// // Extract all fields except 'id' and 'created_at', skip null values
/// let (names, values): (Vec<&str>, Vec<String>) = extract_with_filter(
///     entity.fields(),
///     &["id", "created_at"],
///     true
/// );
/// ```
/// 
/// 从结构体中提取字段名和值，支持过滤选项。
/// 
/// 这是 `extract_with_bind` 的简化版本，提供具有过滤功能的字段提取，
/// 但不包含自定义处理。适用于需要排除某些列或跳过空值的
/// 基本字段提取场景。
/// 
/// # 类型参数
/// * `VAL` - 实现了 `ValueConvert` 的目标值类型
/// 
/// # 参数
/// * `fields` - 字段名及其访问对象的迭代器
/// * `filter_columns` - 要从提取中排除的列名切片
/// * `skip_non_null` - 如果为true，跳过空值或None的字段
/// 
/// # 返回值
/// 包含以下内容的元组：
/// * `Vec<&'static str>` - 提取的字段名向量
/// * `Vec<VAL>` - 转换后的字段值向量
/// 
/// # 示例
/// ```rust
/// use kitx::common::fields::extract_with_filter;
/// 
/// // 提取除 'id' 和 'created_at' 之外的所有字段，跳过空值
/// let (names, values): (Vec<&str>, Vec<String>) = extract_with_filter(
///     entity.fields(),
///     &["id", "created_at"],
///     true
/// );
/// ```
pub fn extract_with_filter<VAL>(
    fields: Fields,
    filter_columns: &[&str],
    skip_non_null: bool
) -> (Vec<&'static str>, Vec<VAL>)
where
    VAL: ValueConvert,
{
    extract_with_bind::<VAL, _>(
        fields,
        filter_columns,
        skip_non_null,
        |_, _|{}
    )
}

/// Extract field data from multiple entities for batch operations.
/// 
/// This function processes multiple entities and extracts their field data
/// in a format suitable for batch database operations like bulk inserts.
/// All entities are expected to have the same field structure, and the
/// field names are extracted from the first entity.
/// 
/// # Type Parameters
/// * `ET` - The entity type that implements `FieldAccess`
/// * `VAL` - The target value type that implements `ValueConvert`
/// 
/// # Arguments
/// * `entities` - Slice of entity references to process
/// * `filter_columns` - Slice of column names to exclude from extraction
/// * `skip_non_null` - If true, skip fields that are empty or None
/// 
/// # Returns
/// A tuple containing:
/// * `Vec<&'static str>` - Vector of field names (from first entity)
/// * `Vec<Vec<VAL>>` - Vector of value vectors, one per entity
/// 
/// # Examples
/// ```rust
/// use kitx::common::fields::batch_extract;
/// 
/// let users = vec![&user1, &user2, &user3];
/// let (field_names, all_values) = batch_extract(
///     &users,
///     &["id"], // Skip ID for insert operations
///     true     // Skip null values
/// );
/// 
/// // field_names: ["name", "email", "age"]
/// // all_values: [["John", "john@example.com", "25"], ...]
/// ```
/// 
/// 从多个实体中提取字段数据用于批量操作。
/// 
/// 此函数处理多个实体并提取它们的字段数据，格式适用于批量数据库操作如批量插入。
/// 所有实体预期具有相同的字段结构，字段名从第一个实体中提取。
/// 
/// # 类型参数
/// * `ET` - 实现了 `FieldAccess` 的实体类型
/// * `VAL` - 实现了 `ValueConvert` 的目标值类型
/// 
/// # 参数
/// * `entities` - 要处理的实体引用切片
/// * `filter_columns` - 要从提取中排除的列名切片
/// * `skip_non_null` - 如果为true，跳过空值或None的字段
/// 
/// # 返回值
/// 包含以下内容的元组：
/// * `Vec<&'static str>` - 字段名向量（来自第一个实体）
/// * `Vec<Vec<VAL>>` - 值向量的向量，每个实体一个
/// 
/// # 示例
/// ```rust
/// use kitx::common::fields::batch_extract;
/// 
/// let users = vec![&user1, &user2, &user3];
/// let (field_names, all_values) = batch_extract(
///     &users,
///     &["id"], // 插入操作时跳过ID
///     true     // 跳过空值
/// );
/// 
/// // field_names: ["name", "email", "age"]
/// // all_values: [["John", "john@example.com", "25"], ...]
/// ```
pub fn batch_extract<ET, VAL>(
    entities: &[&ET], 
    filter_columns: &[&str],
    skip_non_null: bool
) -> (Vec<&'static str>, Vec<Vec<VAL>>)
where 
    ET: FieldAccess,
    VAL: ValueConvert,
{
    let mut entities_names = Vec::new();
    let mut entities_values = Vec::with_capacity(entities.len());

    for entity in entities {
        let value = extract_with_filter::<VAL>(entity.fields(), filter_columns, skip_non_null);
        let (names, values) = value;
        if entities_names.is_empty() {
            entities_names = names;
        }
        entities_values.push(values);
    }

    (entities_names, entities_values)
}

/// Get values for specific columns from an entity.
/// 
/// This function extracts values for a specified list of column names from
/// an entity. If a field doesn't exist or is inaccessible, the default value
/// for the target type is used. This is useful for building WHERE clauses
/// or retrieving specific field values for SQL operations.
/// 
/// # Type Parameters
/// * `ET` - The entity type that implements `FieldAccess`
/// * `VAL` - The target value type that implements `ValueConvert + Default`
/// 
/// # Arguments
/// * `entity` - Reference to the entity to extract values from
/// * `columns` - Slice of column names to extract values for
/// 
/// # Returns
/// A vector of converted values in the same order as the column names.
/// Missing or inaccessible fields will have default values.
/// 
/// # Examples
/// ```rust
/// use kitx::common::fields::get_values;
/// 
/// let user = User { id: 1, name: "John".to_string(), email: Some("john@example.com".to_string()) };
/// let values: Vec<String> = get_values(&user, &["name", "email", "nonexistent"]);
/// // values: ["John", "john@example.com", ""]
/// ```
/// 
/// 从实体中获取特定列的值。
/// 
/// 此函数从实体中提取指定列名列表的值。如果字段不存在或无法访问，
/// 则使用目标类型的默认值。这对于构建WHERE子句或检索SQL操作的
/// 特定字段值很有用。
/// 
/// # 类型参数
/// * `ET` - 实现了 `FieldAccess` 的实体类型
/// * `VAL` - 实现了 `ValueConvert + Default` 的目标值类型
/// 
/// # 参数
/// * `entity` - 要从中提取值的实体引用
/// * `columns` - 要提取值的列名切片
/// 
/// # 返回值
/// 按列名相同顺序的转换值向量。
/// 缺失或无法访问的字段将具有默认值。
/// 
/// # 示例
/// ```rust
/// use kitx::common::fields::get_values;
/// 
/// let user = User { id: 1, name: "John".to_string(), email: Some("john@example.com".to_string()) };
/// let values: Vec<String> = get_values(&user, &["name", "email", "nonexistent"]);
/// // values: ["John", "john@example.com", ""]
/// ```
pub fn get_values<ET, VAL>(
    entity: &ET, 
    columns: &[&str]
) -> Vec<VAL>
where
    ET: FieldAccess,
    VAL: ValueConvert + Default,
{
    let mut values = Vec::new();
    for col in columns {
        let any_value = entity.field_as_any(col);
        if let Some(value) = any_value {
            values.push(VAL::convert(value));
        } else {
            values.push(VAL::default());
        }
    }
    values
}

/// Get a single value for a specific column from an entity.
/// 
/// This function extracts the value for a single column name from an entity.
/// If the field doesn't exist or is inaccessible, the default value for the
/// target type is returned. This is a convenience function for retrieving
/// individual field values, commonly used for primary key extraction or
/// single field operations.
/// 
/// # Type Parameters
/// * `ET` - The entity type that implements `FieldAccess`
/// * `VAL` - The target value type that implements `ValueConvert + Default`
/// 
/// # Arguments
/// * `entity` - Reference to the entity to extract the value from
/// * `column` - Name of the column to extract the value for
/// 
/// # Returns
/// The converted value for the specified column, or the default value
/// if the field is missing or inaccessible.
/// 
/// # Examples
/// ```rust
/// use kitx::common::fields::get_value;
/// 
/// let user = User { id: 1, name: "John".to_string() };
/// let id: i32 = get_value(&user, "id");
/// let name: String = get_value(&user, "name");
/// let missing: String = get_value(&user, "nonexistent"); // Returns ""
/// ```
/// 
/// 从实体中获取特定列的单个值。
/// 
/// 此函数从实体中提取单个列名的值。如果字段不存在或无法访问，
/// 则返回目标类型的默认值。这是一个用于检索单个字段值的便利函数，
/// 常用于主键提取或单字段操作。
/// 
/// # 类型参数
/// * `ET` - 实现了 `FieldAccess` 的实体类型
/// * `VAL` - 实现了 `ValueConvert + Default` 的目标值类型
/// 
/// # 参数
/// * `entity` - 要从中提取值的实体引用
/// * `column` - 要提取值的列名
/// 
/// # 返回值
/// 指定列的转换值，如果字段缺失或无法访问则返回默认值。
/// 
/// # 示例
/// ```rust
/// use kitx::common::fields::get_value;
/// 
/// let user = User { id: 1, name: "John".to_string() };
/// let id: i32 = get_value(&user, "id");
/// let name: String = get_value(&user, "name");
/// let missing: String = get_value(&user, "nonexistent"); // 返回 ""
/// ```
pub fn get_value<ET, VAL>(
    entity: &ET, 
    column: &str,
) -> VAL
where
    ET: FieldAccess,
    VAL: ValueConvert + Default,
{
    entity.field_as_any(column)
        .map(|value| VAL::convert(value))
        .unwrap_or_default()
}
