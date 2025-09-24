//! Common types and structures for database operations
//! 
//! This module defines common data structures and types used throughout the kitx crate
//! for database operations. It includes pagination results, sorting orders, primary key
//! definitions, and cursor-based pagination structures.
//! 
//! 数据库操作的通用类型和结构
//! 
//! 该模块定义了在整个 kitx crate 中用于数据库操作的通用数据结构和类型。
//! 包括分页结果、排序顺序、主键定义和基于游标的分页结构。

use std::fmt::Debug;
use field_access::FieldAccess;
use serde::{Deserialize, Serialize};

use crate::common::{conversion::ValueConvert, fields::get_value};

/// Sort order enum
/// 
/// # Variants
/// * [Asc](Order::Asc) - Ascending order
/// * [Desc](Order::Desc) - Descending order
/// 
/// 排序顺序枚举
/// 
/// # 变体
/// * [Asc](Order::Asc) - 升序
/// * [Desc](Order::Desc) - 降序
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, Hash)]
pub enum Order {
    #[serde(rename = "ASC")]
    #[default]
    Asc,
    #[serde(rename = "DESC")]
    Desc
}

impl Order {
    /// Convert SortOrder to string representation
    /// 
    /// # Returns
    /// Returns "ASC" for ascending order, "DESC" for descending order
    /// 
    /// 将SortOrder转换为字符串表示
    /// 
    /// # 返回值
    /// 升序时返回"ASC"，降序时返回"DESC"
    pub fn as_str(&self) -> &str {
        match self {
            Order::Asc => "ASC",
            Order::Desc => "DESC",
        }
    }
}

/// Join type enum
#[derive(Debug, Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Cross
}

/// Primary key struct
/// 
/// # Variants
/// * [Single](PrimaryKey::Single) - Single column primary key
/// * [Composite](PrimaryKey::Composite) - Composite primary key with multiple columns
/// 
/// 主键结构
/// 
/// # 变体
/// * [Single](PrimaryKey::Single) - 单列主键
/// * [Composite](PrimaryKey::Composite) - 多列组合主键
#[derive(Debug, Clone)]
pub enum PrimaryKey<'a> {
    Single(&'a str, bool),
    Composite(&'a [&'a str]),
}

impl <'a> PrimaryKey<'a> {
    /// Get primary key column names
    /// 
    /// # Returns
    /// A vector containing the names of primary key columns
    /// 
    /// 获取主键列名
    /// 
    /// # 返回值
    /// 包含主键列名的向量
    pub fn get_keys(&self) -> Vec<&'a str> {
        match self {
            PrimaryKey::Single(key, _) => vec![key],
            PrimaryKey::Composite(keys) => keys.iter().map(|x| *x).collect(),
        }
    }
    
    /// Check if the primary key is auto-generated
    /// 
    /// # Returns
    /// True if the primary key is auto-generated, false otherwise
    /// 
    /// 检查主键是否为自动生成
    /// 
    /// # 返回值
    /// 如果主键是自动生成则返回true，否则返回false
    pub fn auto_generate(&self) -> bool {
        match self {
            PrimaryKey::Single(_, flag) => *flag,
            PrimaryKey::Composite(_) => false,
        }
    }
}

/// Paginated query result structure
/// 
/// # Type Parameters
/// * `T` - The type of data records
/// 
/// 分页查询结果结构
/// 
/// # 类型参数
/// * `T` - 数据记录的类型
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, Hash)]
pub struct PaginatedResult<T> {
    /// Data records queried
    /// 
    /// 查询到的数据记录
    pub data: Vec<T>,
    
    /// Total number of records
    /// 
    /// 记录总数
    pub total: u64,
    
    /// Current page number
    /// 
    /// 当前页码
    pub page_number: u64,
    
    /// Number of records per page
    /// 
    /// 每页记录数
    pub page_size: u64,
}

impl<T> PaginatedResult<T> {
    /// Create a new PaginatedResult with the given data, total count, page number, and page size
    /// 
    /// # Arguments
    /// * `data` - The data records
    /// * `total` - Total number of records
    /// * `page_number` - Current page number
    /// * `page_size` - Number of records per page
    /// 
    /// # Returns
    /// A new PaginatedResult instance
    /// 
    /// 使用给定的数据、总数、页码和页面大小创建新的PaginatedResult
    /// 
    /// # 参数
    /// * `data` - 数据记录
    /// * `total` - 记录总数
    /// * `page_number` - 当前页码
    /// * `page_size` - 每页记录数
    /// 
    /// # 返回值
    /// 新的PaginatedResult实例
    pub fn new(data: Vec<T>, total: u64, page_number: u64, page_size: u64) -> Self {
        Self {
            data,
            total,
            page_number,
            page_size,
        }
    }
}

/// Cursor paginated result structure
/// 
/// # Type Parameters
/// * `T` - The type of data records
/// * `C` - The type of cursor value
/// 
/// 游标分页结果结构
/// 
/// # 类型参数
/// * `T` - 数据记录的类型
/// * `C` - 游标值的类型
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq, Hash)]
pub struct CursorPaginatedResult<T, C> {
    /// Data records queried
    /// 
    /// 查询到的数据记录
    pub data: Vec<T>,
    
    /// Next page cursor
    /// 
    /// 下一页游标
    pub next_cursor: Option<C>,
    
    /// Previous page cursor
    /// 
    /// 上一页游标
    pub prev_cursor: Option<C>,
    
    /// Maximum number of records per page
    /// 
    /// 每页最大记录数
    pub limit: u64,
    
    /// Sort order direction
    /// 
    /// 排序方向
    pub sort_order: Order,
}

impl<T, C> CursorPaginatedResult<T, C> {
    /// Create a new CursorPaginatedResult with the given data, limit, and sort order
    /// 
    /// # Arguments
    /// * `data` - The data records
    /// * `limit` - Maximum number of records per page
    /// * `sort_order` - Sort order direction
    /// 
    /// # Returns
    /// A new CursorPaginatedResult instance
    /// 
    /// 使用给定的数据、限制和排序顺序创建新的CursorPaginatedResult
    /// 
    /// # 参数
    /// * `data` - 数据记录
    /// * `limit` - 每页最大记录数
    /// * `sort_order` - 排序方向
    /// 
    /// # 返回值
    /// 新的CursorPaginatedResult实例
    pub fn new(data: Vec<T>, limit: u64, sort_order: Order) -> Self {
        Self {
            data,
            next_cursor: None,
            prev_cursor: None,
            limit,
            sort_order,
        }
    }

    /// Check if there is a next page
    /// 
    /// # Returns
    /// True if there is a next page, false otherwise
    /// 
    /// 检查是否存在下一页
    /// 
    /// # 返回值
    /// 如果存在下一页则返回true，否则返回false
    pub fn has_next_page(&self) -> bool {
        self.next_cursor.is_some() && !self.data.is_empty()
    }

    /// Check if there is a previous page
    /// 
    /// # Returns
    /// True if there is a previous page, false otherwise
    /// 
    /// 检查是否存在上一页
    /// 
    /// # 返回值
    /// 如果存在上一页则返回true，否则返回false
    pub fn has_prev_page(&self) -> bool {
        self.prev_cursor.is_some() && !self.data.is_empty()
    }

    /// Generate cursors for pagination
    /// 
    /// # Type Parameters
    /// * `T` - Must implement FieldAccess trait
    /// * `C` - Must implement ValueConvert and Default traits
    /// 
    /// # Arguments
    /// * `column_key` - The column key to extract cursor values from
    /// 
    /// 为分页生成游标
    /// 
    /// # 类型参数
    /// * `T` - 必须实现FieldAccess trait
    /// * `C` - 必须实现ValueConvert和Default traits
    /// 
    /// # 参数
    /// * `column_key` - 用于提取游标值的列键
    pub fn gen_cursors(&mut self, column_key: &str) 
    where
        T: FieldAccess,
        C: ValueConvert + Default,
    {
        if self.data.len() as u64 == self.limit {
            // 根据排序方向获取双向游标
            let (next_item, prev_item) = match self.sort_order {
                Order::Asc => (self.data.last(), self.data.first()),
                Order::Desc => (self.data.first(), self.data.last()),
            };
            
            self.next_cursor = next_item.map(|item| get_value::<T, C>(item, column_key));
            self.prev_cursor = prev_item.map(|item| get_value::<T, C>(item, column_key));
        }
    }
}