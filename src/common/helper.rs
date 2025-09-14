//! Helper utilities for database operations
//! 
//! This module provides utility functions and structures that support 
//! database operations, including type name conversion and query condition management.
//! 
//! 数据库操作辅助工具
//! 
//! 该模块提供了支持数据库操作的实用函数和结构体，
//! 包括类型名称转换和查询条件管理。

use std::{any::type_name, marker::PhantomData, sync::Arc};

/// Returns the name of the given type
/// 
/// This function converts a Rust type name to a snake_case table name.
/// For example: `ArticleTag` becomes `article_tag`.
/// 
/// # Arguments
/// * `t` - Type to get the name of
/// 
/// # Returns
/// Name of the given type in snake_case format
/// 
/// 返回给定类型的名称
/// 
/// 此函数将 Rust 类型名称转换为 snake_case 表名。
/// 例如：`ArticleTag` 变成 `article_tag`。
/// 
/// # 参数
/// * `t` - 要获取名称的类型
/// 
/// # 返回值
/// 以 snake_case 格式返回给定类型的名称
pub fn get_table_name<T>() -> String {
    let full_name = type_name::<T>();
    let type_name = full_name.rsplit("::").next().unwrap_or(full_name);

    let mut result = String::with_capacity(64);
    let mut chars = type_name.chars().peekable();
    let mut prev_is_lower_or_digit = false;

    while let Some(ch) = chars.next() {
        if ch.is_uppercase() {
            if prev_is_lower_or_digit {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_is_lower_or_digit = true;
        } else if ch.is_lowercase() || ch.is_digit(10) {
            result.push(ch);
            prev_is_lower_or_digit = true;
        } else {
            break;
        }
    }

    result
}

/// A query condition wrapper for concurrent use
/// 
/// This struct wraps query condition closures to enable safe concurrent usage.
/// 
/// # Type Parameters
/// * `Q` - Query type
/// * `F` - Function type that implements the query condition
/// 
/// 查询条件包装器，用于并发复用
/// 
/// 此结构体包装查询条件闭包以实现安全的并发使用。
/// 
/// # 类型参数
/// * `Q` - 查询类型
/// * `F` - 实现查询条件的函数类型
pub struct QueryCondition<'a, Q, F>
where
    F: Fn(&mut Q) + Send + Sync + 'a,
{
    condition: Arc<F>,
    _marker: PhantomData<&'a Q>,
}

impl<'a, Q, F> QueryCondition<'a, Q, F>
where
    F: Fn(&mut Q) + Send + Sync + 'a,
{
    /// Creates a new query condition wrapper
    /// 
    /// # Arguments
    /// * `query_fn` - Closure defining query conditions
    /// 
    /// # Returns
    /// A new QueryCondition instance
    /// 
    /// 创建一个新的查询条件包装器
    /// 
    /// # 参数
    /// * `query_fn` - 定义查询条件的闭包
    /// 
    /// # 返回值
    /// 新的 QueryCondition 实例
    pub fn new(query_fn: F) -> Self {
        QueryCondition {
            condition: Arc::new(query_fn),
            _marker: PhantomData,
        }
    }

    /// Get the query condition closure
    /// 
    /// # Returns
    /// A clone of the wrapped query condition closure
    /// 
    /// 获取查询条件闭包
    /// 
    /// # 返回值
    /// 包装的查询条件闭包的克隆
    pub fn get(&self) -> impl Fn(&mut Q) + Send + Sync + 'a {
        let arc_condition = self.condition.clone();
        move |q| arc_condition(q)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    struct ArticleTag { _tag: String }

    #[test]
    fn test_get_type_name() {
        assert_eq!(get_table_name::<ArticleTag>(), "article_tag");
    }   
}