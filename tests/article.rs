use field_access::FieldAccess;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use kitx::mysql::operations::{DataOperations, EntityOperations};
//use kitx::sqlite::operations::{DataOperations, EntityOperations};
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

impl <'a> From <Article> for EntityOperations<'a, Article> {
    fn from(article: Article) -> Self {
        EntityOperations::new(article, "article", "a_id")
    }
}

impl<'a> ArticleService {
  // 创建EntityOperations
  pub fn as_ops(article: Article) -> EntityOperations<'a, Article> {
      article.into()
  }

  #[allow(dead_code)]
  pub fn by_default() -> EntityOperations<'a, Article> {
      Self::as_ops(Article::default())
  }

  #[allow(dead_code)]
  pub fn by_key(id: i64) -> EntityOperations<'a, Article> {
      Self::as_ops(Article {
          a_id: id,
          ..Default::default()
      })
  }

  #[allow(dead_code)]
  pub fn by_fields(a_class: &str, a_content: &str) -> EntityOperations<'a, Article> {
      Self::as_ops(Article {
          a_class: Some(a_class.to_string()),
          a_content: Some(a_content.to_string()),
          ..Default::default()
      })
  }

}
