use std::marker::PhantomData;

use field_access::FieldAccess;
use sqlx::{Database, Encode, QueryBuilder, Type};

use crate::common::helper::get_table_name;

/// Subquery fragment type: strictly distinguishes between text and binding operations
/// 
/// # Variants
/// * [Text](SubqueryPart::Text) - Text fragment
/// * [Bind](SubqueryPart::Bind) - Binding value
/// 
/// # 中文
/// 子查询片段类型：严格区分文本和绑定操作
/// 
/// # 变体
/// * [Text](SubqueryPart::Text) - 文本片段
/// * [Bind](SubqueryPart::Bind) - 绑定值
enum SubqueryPart<VAL> {
    Text(String),
    Bind(VAL),
}

/// Structure dedicated to nested subqueries
/// 
/// Features:
/// 1. Records operation sequence instead of concrete SQL
/// 2. Does not handle placeholders (determined by parent query)
/// 3. Fully preserves binding values
/// 4. Can only be used nested (no execution methods)
/// 
/// # Type Parameters
/// * `ET` - Entity type that implements FieldAccess and Default traits
/// * `VAL` - Value type
/// 
/// # 中文
/// 专用于嵌套子查询的结构体
/// 
/// 特点：
/// 1. 记录操作序列而非具体SQL
/// 2. 不处理占位符（由父查询决定）
/// 3. 完整保留绑定值
/// 4. 只能被嵌套使用（无执行方法）
/// 
/// # 类型参数
/// * `ET` - 实现 FieldAccess 和 Default traits 的实体类型
/// * `VAL` - 值类型
pub struct Subquery<'a, ET, VAL>
where
    ET: FieldAccess + Default,
    VAL: 'a,
{

    parts: Vec<SubqueryPart<VAL>>,
    has_from: bool,
    _phantom: PhantomData<(ET, &'a ())>,
}

impl<'a, ET, VAL> Subquery<'a, ET, VAL>
where
    ET: FieldAccess + Default,
    VAL: 'a,
{
    /// Add SELECT fields (default all fields)
    /// 
    /// # Returns
    /// A new Subquery instance with default field selection
    /// 
    /// # 中文
    /// 添加 SELECT 字段（默认所有字段）
    /// 
    /// # 返回值
    /// 带有默认字段选择的新 Subquery 实例
    pub fn select_default() -> Self {
        let columns = ET::default().field_names().join(", ");
        Self {
            parts: vec![SubqueryPart::Text(format!("SELECT {}", columns))],
            has_from: false,
            _phantom: PhantomData,
        }
    }

    /// Custom SELECT fields
    /// 
    /// # Arguments
    /// * `f` - Function to build the SELECT fields
    /// 
    /// # Returns
    /// A new Subquery instance with custom field selection
    /// 
    /// # 中文
    /// 自定义 SELECT 字段
    /// 
    /// # 参数
    /// * `f` - 构建 SELECT 字段的函数
    /// 
    /// # 返回值
    /// 带有自定义字段选择的新 Subquery 实例
    pub fn select(f: impl FnOnce(&mut SubqueryBuilder<'_, VAL>)) -> Self {
        let mut parts = vec![SubqueryPart::Text("SELECT ".to_string())];
        let mut builder = SubqueryBuilder { parts: &mut parts };

        f(&mut builder);
        Self {
            parts,
            has_from: false,
            _phantom: PhantomData,
        }
    }

    /// Add table to the subquery
    /// 
    /// # Arguments
    /// * `table_name` - Name of the table to query from
    /// 
    /// # Returns
    /// The Subquery instance with the table added
    /// 
    /// # 中文
    /// 向子查询中添加表
    /// 
    /// # 参数
    /// * `table_name` - 要查询的表名
    /// 
    /// # 返回值
    /// 添加了表的 Subquery 实例
    fn from_table(mut self, table_name: impl Into<String>) -> Self {
        if self.has_from {
            return self;
        }

        self.parts.push(SubqueryPart::Text(format!(" FROM {}", table_name.into())));
        self.has_from = true;
        self
    }

    /// Add default table to the subquery
    /// 
    /// # Returns
    /// The Subquery instance with the default table added
    /// 
    /// # 中文
    /// 向子查询中添加默认表
    /// 
    /// # 返回值
    /// 添加了默认表的 Subquery 实例
    pub fn from_default(self) -> Self {        
        self.from_table(get_table_name::<ET>())
    }

    /// Add table to the subquery, can include alias, between FROM and WHERE
    /// 
    /// # Arguments
    /// * `table_name` - Name of the table to query from, can include alias
    /// 
    /// # Returns
    /// The Subquery instance with the table added
    /// 
    /// # 中文
    /// 向子查询中添加表，可以包含别名，介于 FROM 和 WHERE 之间
    /// 
    /// # 参数
    /// * `table_name` - 要查询的表名，可以包含别名
    /// 
    /// # 返回值
    /// 添加了表的 Subquery 实例
    pub fn from(self, table_name: impl Into<String>) -> Self {
        self.from_table(table_name)
    }

    /// Add WHERE condition (using custom Builder)
    /// 
    /// # Arguments
    /// * `f` - Function to build the WHERE conditions
    /// 
    /// # Returns
    /// The Subquery instance with the WHERE conditions added
    /// 
    /// # 中文
    /// 添加 WHERE 条件（使用自定义 Builder）
    /// 
    /// # 参数
    /// * `f` - 构建 WHERE 条件的函数
    /// 
    /// # 返回值
    /// 添加了 WHERE 条件的 Subquery 实例
    pub fn where_(mut self, f: impl FnOnce(&mut SubqueryBuilder<'_, VAL>)) -> Self {
        if let Some(SubqueryPart::Text(last)) = self.parts.last_mut() {
            if !last.ends_with(' ') {
                *last = format!("{} ", last);
            }
        }
        self.parts.push(SubqueryPart::Text(" WHERE ".to_string()));
 
        let mut builder = SubqueryBuilder { parts: &mut self.parts };
        f(&mut builder);
        self
    }
    
    /// Embed the subquery into the parent query builder
    /// 
    /// # Key Guarantees
    /// 1. Strictly maintains the original operation order
    /// 2. Binding values are added to the parent query in order
    /// 3. Automatically handles parentheses wrapping
    /// 
    /// # Arguments
    /// * `query_builder` - The parent query builder to append to
    /// 
    /// # Type Parameters
    /// * `DB` - Database type that implements sqlx::Database trait
    /// 
    /// # 中文
    /// 将子查询嵌入到父查询构建器中
    /// 
    /// # 关键保证
    /// 1. 严格保持原始操作顺序
    /// 2. 绑定值按序添加到父查询
    /// 3. 自动处理括号包裹
    /// 
    /// # 参数
    /// * `query_builder` - 要追加到的父查询构建器
    /// 
    /// # 类型参数
    /// * `DB` - 实现 sqlx::Database trait 的数据库类型
    pub fn append_to<DB>(self, query_builder: &mut QueryBuilder<'a, DB>)
    where
        VAL: Encode<'a, DB> + Type<DB>,
        DB: Database,
    {
        query_builder.push(" (");

        for part in self.parts {
            match part {
                SubqueryPart::Text(text) => query_builder.push(&text),
                SubqueryPart::Bind(val) => query_builder.push_bind(val),
            };
        }
        
        query_builder.push(") ");
    }
}

/// Builder dedicated to subquery construction
/// 
/// Features:
/// 1. Strictly guarantees operation order
/// 2. Records text and binding values alternately
/// 3. Prevents order confusion during construction
/// 
/// # Type Parameters
/// * `VAL` - Value type
/// 
/// # 中文
/// 专用于子查询构建的 Builder
/// 
/// 特点：
/// 1. 严格保证操作顺序
/// 2. 文本和绑定值交替记录
/// 3. 防止构建时顺序错乱
/// 
/// # 类型参数
/// * `VAL` - 值类型
pub struct SubqueryBuilder<'a, VAL> {
    parts: &'a mut Vec<SubqueryPart<VAL>>,
}

impl<'a, VAL> SubqueryBuilder<'a, VAL> {
    /// Add a text fragment
    /// 
    /// # Arguments
    /// * `s` - Text to add
    /// 
    /// # Returns
    /// A mutable reference to self for chaining
    /// 
    /// # 中文
    /// 添加纯文本片段
    /// 
    /// # 参数
    /// * `s` - 要添加的文本
    /// 
    /// # 返回值
    /// 可变的自身引用，用于链式调用
    pub fn push(&mut self, s: &str) -> &mut Self {
        if let Some(SubqueryPart::Text(last)) = self.parts.last_mut() {
            *last = format!("{}{}", last, s);
            return self;
        }
        self.parts.push(SubqueryPart::Text(s.to_string()));
        self
    }

    /// Add a binding value
    /// 
    /// # Arguments
    /// * `val` - Value to bind
    /// 
    /// # Returns
    /// A mutable reference to self for chaining
    /// 
    /// # 中文
    /// 添加绑定值
    /// 
    /// # 参数
    /// * `val` - 要绑定的值
    /// 
    /// # 返回值
    /// 可变的自身引用，用于链式调用
    pub fn push_bind(&mut self, val: VAL) -> &mut Self {
        self.parts.push(SubqueryPart::Bind(val));
        self
    }
}