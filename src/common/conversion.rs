//! # Value Conversion Module
//! 
//! This module provides traits and utilities for type-safe value conversion
//! and validation, particularly useful for database operations and dynamic
//! type handling.
//! 
//! # 值转换模块
//! 
//! 该模块提供了类型安全的值转换和验证的trait和工具函数，
//! 特别适用于数据库操作和动态类型处理。

use std::any::Any;

/// Trait for converting values to a specific type from a dynamic `Any` reference.
/// 
/// This trait provides a unified interface for converting runtime values to strongly-typed
/// data structures. It's particularly useful in database operations where field values
/// need to be converted from generic containers to specific types.
/// 
/// ## Usage
/// 
/// Implementors should provide logic to safely extract and convert values from the
/// `Any` trait object, handling various input formats including nested `Option` types.
/// 
/// ## Safety
/// 
/// Implementations should handle type conversion errors gracefully and provide
/// sensible defaults when conversion fails.
/// 
/// # 中文
/// 
/// 从动态 `Any` 引用转换值到特定类型的trait。
/// 
/// 该trait提供了将运行时值转换为强类型数据结构的统一接口。
/// 在数据库操作中特别有用，可以将字段值从通用容器转换为特定类型。
/// 
/// ## 使用方法
/// 
/// 实现者应该提供从 `Any` trait对象安全提取和转换值的逻辑，
/// 处理各种输入格式，包括嵌套的 `Option` 类型。
/// 
/// ## 安全性
/// 
/// 实现应该优雅地处理类型转换错误，并在转换失败时提供合理的默认值。
pub trait ValueConvert
{

    /// Converts a dynamic value reference to the implementing type.
    /// 
    /// # Parameters
    /// 
    /// * `value` - A reference to any type that implements `Any`
    /// 
    /// # Returns
    /// 
    /// An instance of the implementing type, constructed from the input value
    /// 
    /// # 中文
    /// 
    /// 将动态值引用转换为实现类型。
    /// 
    /// # 参数
    /// 
    /// * `value` - 实现了 `Any` 的任意类型的引用
    /// 
    /// # 返回值
    /// 
    /// 从输入值构造的实现类型的实例
    fn convert(value: &dyn Any) -> Self;

    /// Checks if a value represents a default primary key value.
    /// 
    /// This method is used to determine whether a value should be considered
    /// a "default" or "empty" primary key, which is useful for distinguishing
    /// between explicitly set values and auto-generated or uninitialized keys.
    /// 
    /// # Parameters
    /// 
    /// * `value` - A reference to the value to check
    /// 
    /// # Returns
    /// 
    /// `true` if the value is considered a default primary key value, `false` otherwise
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// // For integer types, 0 is typically considered default
    /// assert_eq!(SomeType::is_default_value(&0), true);
    /// assert_eq!(SomeType::is_default_value(&42), false);
    /// ```
    /// 
    /// # 中文
    /// 
    /// 检查值是否表示默认的主键值。
    /// 
    /// 此方法用于确定值是否应被视为"默认"或"空"主键，
    /// 这对于区分显式设置的值和自动生成或未初始化的键很有用。
    /// 
    /// # 参数
    /// 
    /// * `value` - 要检查的值的引用
    /// 
    /// # 返回值
    /// 
    /// 如果值被认为是默认主键值则返回 `true`，否则返回 `false`
    /// 
    /// # 示例
    /// 
    /// ```rust,no_run
    /// // 对于整数类型，0通常被认为是默认值
    /// assert_eq!(SomeType::is_default_value(&0), true);
    /// assert_eq!(SomeType::is_default_value(&42), false);
    /// ```
    fn is_default_value(value: &Self) -> bool;
}

/// Helper function to recursively unwrap any number of Option layers
/// and return the inner value if it exists.
/// 
/// This function is particularly useful when dealing with nested Option types
/// that may arise from various data sources, such as database nullable fields
/// or JSON parsing operations. It can handle multiple levels of Option wrapping:
/// 
/// - `Option<Option<T>>` - Nested optional values
/// - `Option<T>` - Single optional values
/// - `T` - Direct values
/// 
/// # Type Parameters
/// 
/// * `T` - The target type to extract from the Option wrapper(s)
/// 
/// # Parameters
/// 
/// * `value` - A reference to any type that implements `Any`
/// 
/// # Returns
/// 
/// * `Some(&T)` - If a value of type T is found within the Option wrapper(s)
/// * `None` - If the value is `None` at any level or cannot be downcast to the target type
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use std::any::Any;
/// 
/// let nested: Option<Option<String>> = Some(Some("Hello".to_string()));
/// let simple: Option<String> = Some("World".to_string());
/// let direct: String = "Direct".to_string();
/// let empty: Option<String> = None;
/// 
/// assert_eq!(unwrap_option::<String>(&nested), Some(&"Hello".to_string()));
/// assert_eq!(unwrap_option::<String>(&simple), Some(&"World".to_string()));
/// assert_eq!(unwrap_option::<String>(&direct), Some(&"Direct".to_string()));
/// assert_eq!(unwrap_option::<String>(&empty), None);
/// ```
/// 
/// # 中文
/// 
/// 递归解包任意数量的Option层并返回内部值（如果存在的话）的辅助函数。
/// 
/// 该函数在处理嵌套Option类型时特别有用，这些类型可能来自各种数据源，
/// 如数据库可空字段或JSON解析操作。它可以处理多层Option包装：
/// 
/// - `Option<Option<T>>` - 嵌套可选值
/// - `Option<T>` - 单一可选值
/// - `T` - 直接值
/// 
/// # 类型参数
/// 
/// * `T` - 从 Option 包装器中提取的目标类型
/// 
/// # 参数
/// 
/// * `value` - 实现了 `Any` 的任意类型的引用
/// 
/// # 返回值
/// 
/// * `Some(&T)` - 如果在 Option 包装器中找到类型 T 的值
/// * `None` - 如果在任意层级上值为 `None` 或无法向下转换为目标类型
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use std::any::Any;
/// 
/// let nested: Option<Option<String>> = Some(Some("Hello".to_string()));
/// let simple: Option<String> = Some("World".to_string());
/// let direct: String = "Direct".to_string();
/// let empty: Option<String> = None;
/// 
/// assert_eq!(unwrap_option::<String>(&nested), Some(&"Hello".to_string()));
/// assert_eq!(unwrap_option::<String>(&simple), Some(&"World".to_string()));
/// assert_eq!(unwrap_option::<String>(&direct), Some(&"Direct".to_string()));
/// assert_eq!(unwrap_option::<String>(&empty), None);
/// ```
pub fn unwrap_option<T: 'static>(value: &dyn Any) -> Option<&T> {
    if let Some(opt_opt) = value.downcast_ref::<Option<Option<T>>>() {
        return opt_opt.as_ref().and_then(|opt| opt.as_ref());
    }
    if let Some(opt) = value.downcast_ref::<Option<T>>() {
        return opt.as_ref();
    }
    value.downcast_ref::<T>()
}

/// Helper function to check if a value is empty and handle Option types using a closure.
/// 
/// This function provides a comprehensive way to determine if a value should be
/// considered "empty" or "absent". It supports multiple types and handles nested
/// Option types gracefully. The function is particularly useful for:
/// 
/// - Database field validation
/// - Input sanitization
/// - Default value determination
/// - Conditional processing based on data presence
/// 
/// ## Supported Types
/// 
/// The function can check emptiness for:
/// 
/// - **String types**: `String`, `&str` - empty strings or strings equal to "null" (case-insensitive)
/// - **Binary types**: `Vec<u8>`, `&[u8]` - empty byte arrays
/// - **Option types**: Any level of Option nesting, including `Option<Option<T>>`
/// - **Unit type**: `Option<()>` - checks if None
/// 
/// ## Option Handling
/// 
/// The function can handle multiple levels of Option wrapping:
/// - `Option<Option<T>>` - Returns true if None at any level or if inner value is empty
/// - `Option<T>` - Returns true if None or if inner value is empty
/// - `T` - Returns true if value is considered empty for its type
/// 
/// # Parameters
/// 
/// * `value` - A reference to any type that implements `Any`
/// 
/// # Returns
/// 
/// * `true` - If the value is empty, None, or represents an absent value
/// * `false` - If the value contains meaningful data
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use std::any::Any;
/// 
/// // String examples
/// assert_eq!(is_empty_or_none(&""), true);                    // Empty string
/// assert_eq!(is_empty_or_none(&"null"), true);               // Null string (case-insensitive)
/// assert_eq!(is_empty_or_none(&"Hello"), false);             // Non-empty string
/// 
/// // Option examples
/// let none_str: Option<String> = None;
/// let some_empty = Some("".to_string());
/// let some_value = Some("Hello".to_string());
/// 
/// assert_eq!(is_empty_or_none(&none_str), true);             // None option
/// assert_eq!(is_empty_or_none(&some_empty), true);           // Some with empty value
/// assert_eq!(is_empty_or_none(&some_value), false);          // Some with value
/// 
/// // Binary examples
/// let empty_vec: Vec<u8> = vec![];
/// let filled_vec = vec![1, 2, 3];
/// 
/// assert_eq!(is_empty_or_none(&empty_vec), true);            // Empty vector
/// assert_eq!(is_empty_or_none(&filled_vec), false);          // Non-empty vector
/// ```
/// 
/// # 中文
/// 
/// 检查值是否为空并使用闭包处理Option类型的辅助函数。
/// 
/// 该函数提供了一种全面的方式来确定值是否应被认为“空”或“缺失”。
/// 它支持多种类型并优雅地处理嵌套Option类型。该函数特别适用于：
/// 
/// - 数据库字段验证
/// - 输入清理
/// - 默认值确定
/// - 基于数据存在性的条件处理
/// 
/// ## 支持的类型
/// 
/// 该函数可以检查以下类型的空值：
/// 
/// - **字符串类型**: `String`、`&str` - 空字符串或等于"null"的字符串（不区分大小写）
/// - **二进制类型**: `Vec<u8>`、`&[u8]` - 空字节数组
/// - **Option类型**: 任意层级的Option嵌套，包括 `Option<Option<T>>`
/// - **单元类型**: `Option<()>` - 检查是否为None
/// 
/// ## Option处理
/// 
/// 该函数可以处理多层Option包装：
/// - `Option<Option<T>>` - 如果在任意层级为None或内部值为空则返回true
/// - `Option<T>` - 如果为None或内部值为空则返回true
/// - `T` - 如果该类型的值被认为空则返回true
/// 
/// # 参数
/// 
/// * `value` - 实现了 `Any` 的任意类型的引用
/// 
/// # 返回值
/// 
/// * `true` - 如果值为空、None或表示缺失的值
/// * `false` - 如果值包含有意义的数据
/// 
/// # 示例
/// 
/// ```rust,no_run
/// use std::any::Any;
/// 
/// // 字符串示例
/// assert_eq!(is_empty_or_none(&""), true);                    // 空字符串
/// assert_eq!(is_empty_or_none(&"null"), true);               // Null字符串（不区分大小写）
/// assert_eq!(is_empty_or_none(&"Hello"), false);             // 非空字符串
/// 
/// // Option示例
/// let none_str: Option<String> = None;
/// let some_empty = Some("".to_string());
/// let some_value = Some("Hello".to_string());
/// 
/// assert_eq!(is_empty_or_none(&none_str), true);             // None option
/// assert_eq!(is_empty_or_none(&some_empty), true);           // 包含空值的Some
/// assert_eq!(is_empty_or_none(&some_value), false);          // 包含值的Some
/// 
/// // 二进制示例
/// let empty_vec: Vec<u8> = vec![];
/// let filled_vec = vec![1, 2, 3];
/// 
/// assert_eq!(is_empty_or_none(&empty_vec), true);            // 空向量
/// assert_eq!(is_empty_or_none(&filled_vec), false);          // 非空向量
/// ```
pub fn is_empty_or_none(value: &dyn Any) -> bool {
    /// Internal macro to check if a specific type is empty or None
    /// 内部宏，用于检查特定类型是否为空或None
    macro_rules! check_type {
        ($ty:ty, $predicate:expr) => {{
            // Check for nested Option<Option<T>>
            // 检查嵌套的 Option<Option<T>>
            if let Some(opt) = value.downcast_ref::<Option<Option<$ty>>>() {
                return opt.as_ref().map_or(true, |v| v.as_ref().map_or(true, $predicate));
            }

            // Check for single Option<T>
            // 检查单一的 Option<T>
            if let Some(opt) = value.downcast_ref::<Option<$ty>>() {
                return opt.as_ref().map_or(true, $predicate);
            }

            // Check for direct type T
            // 检查直接类型 T
            if let Some(v) = value.downcast_ref::<$ty>() {
                return $predicate(v);
            }
        }};
    }

    // Check string types (String and &str) for emptiness or "null" values
    // 检查字符串类型（String 和 &str）是否为空或"null"值
    check_type!(String, |s: &String| s.is_empty() || s.eq_ignore_ascii_case("null"));
    check_type!(&str, |s: &&str| s.is_empty() || s.eq_ignore_ascii_case("null"));
    
    // Check binary types (Vec<u8> and &[u8]) for emptiness
    // 检查二进制类型（Vec<u8> 和 &[u8]）是否为空
    check_type!(Vec<u8>, |b: &Vec<u8>| b.is_empty());
    check_type!(&[u8], |b: &&[u8]| b.is_empty());

    // Special case for Option<()> - check if None
    // Option<()> 的特殊情况 - 检查是否为 None
    if let Some(opt) = value.downcast_ref::<Option<()>>() {
        return opt.is_none();
    }
    
    // If none of the above types match, consider the value as non-empty
    // 如果以上类型都不匹配，则认为值不为空
    false
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap_option() {
        let opt_opt = Some(Some("Hello".to_string()));
        let opt = Some("World".to_string());
        let direct = "Direct".to_string();
        let opt_none: Option<Option<String>> = None;
        let opt_inner_none: Option<Option<String>> = Some(None);

        assert_eq!(unwrap_option(&opt_opt), Some(&"Hello".to_string()));
        assert_eq!(unwrap_option(&opt), Some(&"World".to_string()));
        assert_eq!(unwrap_option(&direct), Some(&"Direct".to_string()));
        assert_eq!(unwrap_option::<String>(&opt_none), None);
        assert_eq!(unwrap_option::<String>(&opt_inner_none), None);
    }


    #[test]
    fn test_check_empty_or_none() {
        let str = "Hello";
        let empty_str = "";
        let null_str = "null";
        let null_upper = "NULL";

        let opt_str = Some("World".to_string());
        let opt_none: Option<String> = None;
        let empty_opt_str = Some("".to_string());
        let null_opt_str = Some("null".to_string());
        
        let nested_some = Some(Some("Hello".to_string()));
        let nested_none: Option<Option<String>> = None;
        let nested_inner_none: Option<Option<String>> = Some(None);
        let nested_empty = Some(Some("".to_string()));
        
        let empty_vec: Vec<u8> = vec![];
        let filled_vec = vec![1, 2, 3];
        let empty_slice: &[u8] = &[];
        let filled_slice: &[u8] = &[1, 2, 3];

        assert!(!is_empty_or_none(&str));          // Non-empty string should be false
        assert!(is_empty_or_none(&empty_str));     // Empty string should be true
        assert!(is_empty_or_none(&null_str));      // "null" string should be true
        assert!(is_empty_or_none(&null_upper));    // "NULL" string should be true
        
        assert!(!is_empty_or_none(&opt_str));      // Some with value should be false
        assert!(is_empty_or_none(&opt_none));      // None should be true
        assert!(is_empty_or_none(&empty_opt_str)); // Some with empty string should be true
        assert!(is_empty_or_none(&null_opt_str));  // Some with "null" should be true

        assert!(!is_empty_or_none(&nested_some));     // Some(Some(value)) should be false
        assert!(is_empty_or_none(&nested_none));      // None should be true
        assert!(is_empty_or_none(&nested_inner_none)); // Some(None) should be true
        assert!(is_empty_or_none(&nested_empty));     // Some(Some("")) should be true

        assert!(is_empty_or_none(&empty_vec));     // Empty vector should be true
        assert!(!is_empty_or_none(&filled_vec));   // Non-empty vector should be false
        assert!(is_empty_or_none(&empty_slice));   // Empty slice should be true
        assert!(!is_empty_or_none(&filled_slice)); // Non-empty slice should be false       
    }
    
}