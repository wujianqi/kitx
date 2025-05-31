pub mod global;
pub mod connection;
pub mod kind;
pub mod query;
pub mod single;
pub mod composite;

use crate::sql::{
    query_builder::SqlBuilder, 
    delete::DeleteBuilder, 
    insert::InsertBuilder, 
    select::SelectBuilder, 
    update::UpdateBuilder
};
use kind::DataKind;

pub type Sql<'a> = SqlBuilder<DataKind<'a>>;
pub type Select<'a> = SelectBuilder<DataKind<'a>>;
pub type Insert<'a> = InsertBuilder<DataKind<'a>>;
pub type Update<'a> = UpdateBuilder<DataKind<'a>>;
pub type Delete<'a> = DeleteBuilder<DataKind<'a>>;
