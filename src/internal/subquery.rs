use std::marker::PhantomData;

use field_access::FieldAccess;
use sqlx::{Database, Encode, QueryBuilder, Type};

use crate::common::{helper::get_table_name, types::JoinType};

/// Subquery fragment type: strictly distinguishes between text and binding operations
/// 
/// # Variants
/// * [Text](SubqueryPart::Text) - Text fragment
/// * [Bind](SubqueryPart::Bind) - Binding value
/// 
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
    table_name: String,
    has_from: bool,
    has_filter: bool,
    has_group_by: bool,
    has_having: bool,
    _phantom: PhantomData<(ET, &'a ())>,
}

impl<'a, ET, VAL> Subquery<'a, ET, VAL>
where
    ET: FieldAccess + Default,
    VAL: 'a,
{
    pub fn table() -> Self {
        Self::with_table(&get_table_name::<ET>())
    }

    /// 开始构建 SELECT 查询（指定表名）
    pub fn with_table(table_name: &str) -> Self {
        Self {
            parts: vec![SubqueryPart::Text("SELECT ".to_string())],
            table_name: table_name.to_string(),
            has_from: false,  
            has_filter: false, 
            has_group_by: false,
            has_having: false,
            _phantom: PhantomData,
        }
    }

    fn push_part(&mut self, f: impl FnOnce(&mut SubqueryBuilder<'_, VAL>)) {
        if let Some(SubqueryPart::Text(last)) = self.parts.last_mut() {
            if !last.ends_with(' ') {
                *last = format!("{} ", last);
            }
        } 
        let mut builder = SubqueryBuilder { parts: &mut self.parts };
        f(&mut builder)
    }

    /// 添加自定义列
    pub fn columns(
        mut self,
        column_build_fn: impl FnOnce(&mut SubqueryBuilder<'_, VAL>)
    ) -> Self {
        if self.has_from {
            return self;
        }

        self.push_part(column_build_fn);
        self.parts.push(SubqueryPart::Text(format!(" FROM {}", &self.table_name)));
        self.has_from = true;
        self
    }

    /// 添加所有字段
    fn add_from_clause(&mut self) {        
        let columns = ET::default().field_names().join(", ");
        self.parts.push(SubqueryPart::Text(format!("{} FROM {}", columns, &self.table_name)));
        self.has_from = true;
    }

    
    /// Add WHERE condition (using custom Builder)
    /// 
    /// # Arguments
    /// * `f` - Function to build the WHERE conditions
    /// 
    /// # Returns
    /// The Subquery instance with the WHERE conditions added
    /// 
    /// 添加 WHERE 条件（使用自定义 Builder）
    /// 
    /// # 参数
    /// * `f` - 构建 WHERE 条件的函数
    /// 
    /// # 返回值
    /// 添加了 WHERE 条件的 Subquery 实例
    pub fn filter(mut self, f: impl FnOnce(&mut SubqueryBuilder<'_, VAL>)) -> Self {
        if !self.has_from {
            self.add_from_clause();
        }
        if !self.has_filter {
            self.parts.push(SubqueryPart::Text(" WHERE ".to_string()));
        }       
        self.push_part(f);
        self
    }

    /// Add JOIN clause
    pub fn join(
        mut self,
        join_type: JoinType,
        table: impl Into<String>,
        on_condition: impl FnOnce(&mut SubqueryBuilder<'_, VAL>),
    ) -> Self {
        if !self.has_from {
            self.add_from_clause();
        }
        let join_keyword = match join_type {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL JOIN",
            JoinType::Cross => "CROSS JOIN",
        };

        self.parts.push(SubqueryPart::Text(table.into()));
        self.parts.push(SubqueryPart::Text(join_keyword.to_string()));
        self.push_part(on_condition);
        self
    }

    /// 添加 GROUP BY 子句
    /// 
    /// # Arguments
    /// * `field` - 分组字段（可为表达式）
    /// 
    /// # Returns
    pub fn group_by(mut self, field: impl Into<String>) -> Self {
        if !self.has_from {
            self.add_from_clause();
        }
        let field = field.into();
      
        if self.has_group_by {
            self.parts.push(SubqueryPart::Text(", ".into()));
            
        } else {
            self.parts.push(SubqueryPart::Text(" GROUP BY ".into()));
            self.has_group_by = true;
        }
        self.parts.push(SubqueryPart::Text(field.into()));
        
        self
    }

    /// 添加 HAVING 子句（必须在 GROUP BY 之后）
    /// 
    /// # Arguments
    /// * `condition` - HAVING 条件构建函数
    /// 
    /// # Returns
    /// 添加了 HAVING 的 Select 实例   
    pub fn having(
        mut self,
        condition: impl FnOnce(&mut SubqueryBuilder<'_, VAL>),
    ) -> Self {
        if !self.has_group_by {
            return self;
        }
        if !self.has_having {
            self.parts.push(SubqueryPart::Text(" HAVING ".into()));
            self.has_having = true;
        }        
        self.push_part(condition);
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
    pub fn append_to<DB>(mut self, query_builder: &mut QueryBuilder<'a, DB>)
    where
        VAL: Encode<'a, DB> + Type<DB>,
        DB: Database,
    {
        query_builder.push(" (");

        if !self.has_from {
            self.add_from_clause();
        }
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

