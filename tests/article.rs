use field_access::FieldAccess;
use kitx::common::operations::OperationsTrait;
//use kitx::sqlite::operations::Operations;
use kitx::mysql::operations::Operations;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
//use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Default, FromRow, FieldAccess, Clone)]
pub struct Article {
    pub a_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a_content: Option<String>,
}

#[allow(dead_code)]
pub struct ArticleService;

impl<'a> ArticleService {
  // 创建EntityOperations
  pub fn new() -> Operations<'a, Article> {
    Operations::new("article", "a_id", None)
  }
}
