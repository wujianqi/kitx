pub use crate::common::types::{OrderBy, PrimaryKey, CursorPaginatedResult, PaginatedResult};
pub use crate::common::builder::{BuilderTrait, FilterTrait};
pub use crate::common::error::{KitxError, QueryError, SoftDeleteError, RelationError};
pub use crate::common::query::QueryExecutor;
pub use crate::common::operations::{OpsBuilderTrait, OpsActionTrait};
pub use crate::common::relation::EntitiesRelation;
pub use crate::sql::filter::{Expr, ColumnExpr};
pub use crate::sql::{agg::Func, case_when::CaseWhen, cte::{CTE, WithCTE},  join::JoinType};

#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use crate::sqlite::{
        connection::{create_db_pool, setup_db_pool},
        global::{get_global_filter, set_global_filter, get_global_soft_delete_field, set_global_soft_delete_field},
        kind::DataKind,
        single::Operations,
        composite::Operations as MutliKeyOperations,
        query::SqliteQuery as Query,
        Sql, Select, Update, Insert, Delete,
    };
}

#[cfg(feature = "mysql")]
pub mod mysql {
    pub use crate::mysql::{
        connection::{create_db_pool, setup_db_pool},
        global::{get_global_filter, set_global_filter, get_global_soft_delete_field, set_global_soft_delete_field},
        kind::DataKind,
        single::Operations,
        composite::Operations as MutliKeyOperations,
        query::MySqlQuery as Query,
        Sql, Select, Update, Insert, Delete,
    };
}

#[cfg(feature = "postgres")]
pub mod postgres {
    pub use crate::postgres::{
        connection::{create_db_pool, setup_db_pool},
        global::{get_global_filter, set_global_filter, get_global_soft_delete_field, set_global_soft_delete_field},
        kind::DataKind,
        single::Operations,
        composite::Operations as MutliKeyOperations,
        query::PostgresQuery as Query,
        Sql, Select, Update, Insert, Delete,
    };
}