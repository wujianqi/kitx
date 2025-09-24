pub use crate::common::types::{Order, PrimaryKey, CursorPaginatedResult, PaginatedResult};
pub use crate::common::error::{KitxError, QueryError, RelationError};
pub use crate::common::fields::{batch_extract, extract_all, extract_with_bind, extract_with_filter, get_value, get_values};
pub use crate::common::filter::{push_primary_key_bind, push_primary_key_conditions};
pub use crate::common::helper::{get_table_name, QueryCondition};
pub use crate::common::relation::EntitiesRelation;

#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use crate::sqlite::{
        connection::{create_db_pool, setup_db_pool},
        kind::DataKind,
        query::{execute, execute_with_trans, fetch_all, fetch_one, fetch_optional, fetch_scalar, fetch_scalar_optional},
        builder::{Insert, Select, Update, Delete, Upsert, Subquery, QB, SQB},
    };
}

#[cfg(feature = "mysql")]
pub mod mysql {
    pub use crate::mysql::{
        connection::{create_db_pool, setup_db_pool},
        kind::DataKind,
        query::{execute, execute_with_trans, fetch_all, fetch_one, fetch_optional, fetch_scalar, fetch_scalar_optional},
        builder::{Insert, Select, Update, Delete, Upsert, Subquery, QB, SQB},
    };
}

#[cfg(feature = "postgres")]
pub mod postgres {
    pub use crate::postgres::{
        connection::{create_db_pool, setup_db_pool},
        kind::DataKind,
        query::{execute, execute_with_trans, fetch_all, fetch_one, fetch_optional, fetch_scalar, fetch_scalar_optional},
        builder::{Insert, Select, Update, Delete, Upsert, Subquery, QB, SQB},
    };
}