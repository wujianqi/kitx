//! Error handling module for Kitx database operations.
//! 
//! This module provides comprehensive error types for database operations,
//! including query errors, relation errors, and general Kitx errors.
//! All errors implement proper error traits and can be converted between
//! different error types for seamless integration with sqlx.
//! 
//! # 中文
//! Kitx数据库操作的错误处理模块。
//! 
//! 此模块为数据库操作提供全面的错误类型，包括查询错误、关联错误和通用Kitx错误。
//! 所有错误都实现了适当的错误trait，并可以在不同错误类型之间转换，
//! 以便与sqlx无缝集成。

use std::fmt::{Display, Formatter, Result, Debug};
use std::error::Error;
use sqlx::error::{DatabaseError, ErrorKind};
use sqlx::Error as SqlxError;

/// Main error type for Kitx operations.
/// 
/// This struct wraps error messages and implements the necessary traits
/// to integrate with Rust's error handling system and sqlx's DatabaseError trait.
/// 
/// # Examples
/// ```rust
/// use kitx::common::error::KitxError;
/// 
/// let error = KitxError::new("Database connection failed".to_string());
/// println!("Error: {}", error);
/// ```
/// 
/// # 中文
/// Kitx操作的主要错误类型。
/// 
/// 此结构体包装错误消息，并实现必要的trait以与Rust的错误处理系统
/// 和sqlx的DatabaseError trait集成。
/// 
/// # 示例
/// ```rust
/// use kitx::common::error::KitxError;
/// 
/// let error = KitxError::new("数据库连接失败".to_string());
/// println!("错误: {}", error);
/// ```
#[derive(Debug)]
pub struct KitxError {
    message: String,
}

/// Query-specific error types for database operations.
/// 
/// This enum covers various error scenarios that can occur during
/// database query construction and execution, providing specific
/// error types for better error handling and debugging.
/// 
/// # Variants
/// - `DBPoolNotInitialized`: Database connection pool is not initialized
/// - `NoPrimaryKeyDefined`: Primary key is not defined for the entity
/// - `PageNumberInvalid`: Invalid pagination parameters
/// - `LimitInvalid`: Invalid limit value for queries
/// - `ColumnsListEmpty`: No valid columns provided for the operation
/// - `NoEntitiesProvided`: No entities provided for batch operations
/// - `ValueInvalid`: Invalid value for a specific column
/// - `DuplicateWhereClause`: Duplicate WHERE clause detected
/// - `Other`: Generic error with custom message
/// 
/// # 中文
/// 数据库操作的查询特定错误类型。
/// 
/// 此枚举涵盖数据库查询构建和执行过程中可能发生的各种错误场景，
/// 为更好的错误处理和调试提供特定的错误类型。
/// 
/// # 变体
/// - `DBPoolNotInitialized`: 数据库连接池未初始化
/// - `NoPrimaryKeyDefined`: 实体未定义主键
/// - `PageNumberInvalid`: 无效的分页参数
/// - `LimitInvalid`: 查询的限制值无效
/// - `ColumnsListEmpty`: 操作未提供有效列
/// - `NoEntitiesProvided`: 批量操作未提供实体
/// - `ValueInvalid`: 特定列的值无效
/// - `DuplicateWhereClause`: 检测到重复的WHERE子句
/// - `Other`: 带有自定义消息的通用错误
#[derive(Debug)]
pub enum QueryError {
    /// Database pool is not initialized / 数据库连接池未初始化
    DBPoolNotInitialized,
    /// No primary key defined for the entity / 实体未定义主键
    NoPrimaryKeyDefined,
    /// Page number or page size is invalid / 页码或页面大小无效
    PageNumberInvalid,
    /// Limit value is invalid / 限制值无效
    LimitInvalid,
    /// No valid columns provided / 未提供有效列
    ColumnsListEmpty,
    /// No entities provided for batch operation / 批量操作未提供实体
    NoEntitiesProvided,
    /// Invalid value for the specified column / 指定列的值无效
    ValueInvalid(String),
    /// Duplicate WHERE clause detected / 检测到重复的WHERE子句
    DuplicateWhereClause,
    /// Generic error with custom message / 带有自定义消息的通用错误
    Other(String),
}

/// Relation-specific error types for handling entity relationships.
/// 
/// This enum handles errors that occur when working with entity relationships,
/// such as foreign key constraints and value matching between related entities.
/// 
/// # Variants
/// - `ValueEmpty`: Expected non-empty values but got empty collection
/// - `ValueMismatch`: Value type or content mismatch between expected and actual
/// 
/// # 中文
/// 处理实体关系的关联特定错误类型。
/// 
/// 此枚举处理在处理实体关系时发生的错误，
/// 如外键约束和相关实体之间的值匹配。
/// 
/// # 变体
/// - `ValueEmpty`: 期望非空值但得到空集合
/// - `ValueMismatch`: 期望值与实际值的类型或内容不匹配
#[derive(Debug)]
pub enum RelationError {
    /// Expected non-empty values but got empty collection / 期望非空值但得到空集合
    ValueEmpty(usize),
    /// Value mismatch between expected and actual / 期望值与实际值不匹配
    ValueMismatch(usize, String, String),
}

impl QueryError {
    /// Returns a descriptive error message for the query error.
    /// 
    /// This method provides human-readable error messages that can be
    /// displayed to users or logged for debugging purposes.
    /// 
    /// # Returns
    /// A `String` containing the error description.
    /// 
    /// # Examples
    /// ```rust
    /// use kitx::common::error::QueryError;
    /// 
    /// let error = QueryError::DBPoolNotInitialized;
    /// assert_eq!(error.message(), "Database pool not initialized");
    /// ```
    /// 
    /// # 中文
    /// 返回查询错误的描述性错误消息。
    /// 
    /// 此方法提供可读的错误消息，可以显示给用户或记录用于调试。
    /// 
    /// # 返回值
    /// 包含错误描述的 `String`。
    /// 
    /// # 示例
    /// ```rust
    /// use kitx::common::error::QueryError;
    /// 
    /// let error = QueryError::DBPoolNotInitialized;
    /// assert_eq!(error.message(), "Database pool not initialized");
    /// ```
    pub fn message(&self) -> String {
        match self {
            Self::DBPoolNotInitialized => "Database pool not initialized".to_string(),
            Self::NoPrimaryKeyDefined => "No primary key defined".to_string(),
            Self::PageNumberInvalid => "Page number and page size must be greater than 0".to_string(),
            Self::LimitInvalid => "Limit must be greater than 0".to_string(),
            Self::ValueInvalid(column_name) => format!("Field {} has an invalid value", column_name),
            Self::ColumnsListEmpty => "No valid fields provided".to_string(),
            Self::NoEntitiesProvided => "No entities provided".to_string(),
            Self::DuplicateWhereClause => "Duplicate WHERE clause".to_string(),
            Self::Other(msg) => msg.to_owned(),
        }
    }
}

impl RelationError {
    /// Returns a descriptive error message for the relation error.
    /// 
    /// This method provides detailed error messages for relationship-related
    /// errors, including context about the specific values or indices involved.
    /// 
    /// # Returns
    /// A `String` containing the error description with relevant context.
    /// 
    /// # Examples
    /// ```rust
    /// use kitx::common::error::RelationError;
    /// 
    /// let error = RelationError::ValueEmpty(0);
    /// assert_eq!(error.message(), "Expected non-empty values, got 0");
    /// ```
    /// 
    /// # 中文
    /// 返回关联错误的描述性错误消息。
    /// 
    /// 此方法为关系相关错误提供详细的错误消息，
    /// 包括涉及的特定值或索引的上下文。
    /// 
    /// # 返回值
    /// 包含错误描述和相关上下文的 `String`。
    /// 
    /// # 示例
    /// ```rust
    /// use kitx::common::error::RelationError;
    /// 
    /// let error = RelationError::ValueEmpty(0);
    /// assert_eq!(error.message(), "Expected non-empty values, got 0");
    /// ```
    pub fn message(&self) -> String {
        match self {
            Self::ValueEmpty(size) => format!("Expected non-empty values, got {}", size),
            Self::ValueMismatch(index, expected, actual) => 
                format!("Value mismatch: index {}, expected {}, got {}", index, expected, actual),
        }
    }
}

impl Display for KitxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.message)
    }
}

impl Error for KitxError {}

impl KitxError { 

    /// Creates a new KitxError instance
    /// 
    /// # Arguments
    /// * `message` - Error description message
    /// # 中文
    /// 创建一个新的KitxError实例
    /// 
    /// # 参数
    /// * `message` - 错误描述信息
    pub fn new(message: String) -> Self {
        KitxError { message }
    }
}

impl From<QueryError> for KitxError {

    /// Converts a QueryError into a KitxError.
    /// 
    /// This conversion allows QueryError instances to be used wherever
    /// KitxError is expected, providing seamless error type conversion.
    /// 
    /// # Arguments
    /// * `err` - The QueryError to convert
    /// 
    /// # Returns
    /// A new KitxError instance with the error message from the QueryError.
    /// 
    /// # 中文
    /// 将QueryError转换为KitxError。
    /// 
    /// 此转换允许在期望KitxError的地方使用QueryError实例，
    /// 提供无缝的错误类型转换。
    /// 
    /// # 参数
    /// * `err` - 要转换的QueryError
    /// 
    /// # 返回值
    /// 带有来自QueryError的错误消息的新KitxError实例。
    fn from(err: QueryError) -> Self {
        KitxError {  message: err.message() }
    }
}

impl From<QueryError> for SqlxError {
    /// Converts a QueryError into a SqlxError.
    /// 
    /// This conversion enables QueryError to be used in sqlx contexts
    /// by wrapping it in a KitxError and then converting to SqlxError.
    /// 
    /// # Arguments
    /// * `err` - The QueryError to convert
    /// 
    /// # Returns
    /// A SqlxError containing the QueryError wrapped in a KitxError.
    /// 
    /// # 中文
    /// 将QueryError转换为SqlxError。
    /// 
    /// 此转换通过将QueryError包装在KitxError中然后转换为SqlxError，
    /// 使QueryError能够在sqlx上下文中使用。
    /// 
    /// # 参数
    /// * `err` - 要转换的QueryError
    /// 
    /// # 返回值
    /// 包含封装在KitxError中的QueryError的SqlxError。
    fn from(err: QueryError) -> Self {
        SqlxError::Database(Box::new(KitxError {  message: err.message() }))
    }
}

impl From<RelationError> for SqlxError {
    /// Converts a RelationError into a SqlxError.
    /// 
    /// This conversion enables RelationError to be used in sqlx contexts
    /// by wrapping it in a KitxError and then converting to SqlxError.
    /// 
    /// # Arguments
    /// * `err` - The RelationError to convert
    /// 
    /// # Returns
    /// A SqlxError containing the RelationError wrapped in a KitxError.
    /// 
    /// # 中文
    /// 将RelationError转换为SqlxError。
    /// 
    /// 此转换通过将RelationError包装在KitxError中然后转换为SqlxError，
    /// 使RelationError能够在sqlx上下文中使用。
    /// 
    /// # 参数
    /// * `err` - 要转换的RelationError
    /// 
    /// # 返回值
    /// 包含封装在KitxError中的RelationError的SqlxError。
    fn from(err: RelationError) -> Self {
        SqlxError::Database(Box::new(KitxError { message: err.message() }))
    }
}

impl DatabaseError for KitxError {
    /// Returns a reference to the error as a trait object.
    /// 
    /// # Returns
    /// A reference to self as an Error trait object.
    /// 
    /// # 中文
    /// 返回错误作为trait对象的引用。
    /// 
    /// # 返回值
    /// self作为Error trait对象的引用。
    fn as_error(&self) -> &(dyn Error + Send + Sync + 'static) {
        self
    }
    
    /// Returns the error message.
    /// 
    /// # Returns
    /// A string slice containing the error message.
    /// 
    /// # 中文
    /// 返回错误消息。
    /// 
    /// # 返回值
    /// 包含错误消息的字符串切片。
    fn message(&self) -> &str {
        &self.message
    }
    
    /// Returns a mutable reference to the error as a trait object.
    /// 
    /// # Returns
    /// A mutable reference to self as an Error trait object.
    /// 
    /// # 中文
    /// 返回错误作为trait对象的可变引用。
    /// 
    /// # 返回值
    /// self作为Error trait对象的可变引用。
    fn as_error_mut(&mut self) -> &mut (dyn Error + Send + Sync + 'static) {
        self
    }

    /// Converts the boxed error into a boxed trait object.
    /// 
    /// # Arguments
    /// * `self` - The boxed KitxError to convert
    /// 
    /// # Returns
    /// A boxed Error trait object.
    /// 
    /// # 中文
    /// 将装箱的错误转换为装箱的trait对象。
    /// 
    /// # 参数
    /// * `self` - 要转换的装箱KitxError
    /// 
    /// # 返回值
    /// 装箱的Error trait对象。
    fn into_error(self: Box<Self>) -> Box<dyn Error + Send + Sync + 'static> {
        self
    }

    /// Returns the kind of database error.
    /// 
    /// All KitxError instances are classified as "Other" error kind
    /// since they represent custom application-level errors.
    /// 
    /// # Returns
    /// Always returns `ErrorKind::Other`.
    /// 
    /// # 中文
    /// 返回数据库错误的类型。
    /// 
    /// 所有KitxError实例都被分类为"Other"错误类型，
    /// 因为它们表示自定义的应用程序级错误。
    /// 
    /// # 返回值
    /// 总是返回 `ErrorKind::Other`。
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}