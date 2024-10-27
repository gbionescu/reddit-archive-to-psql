use crate::service::InsertStrategy;
use sqlx::PgPool;

pub trait DBTable {
    fn table_name() -> &'static str;
    fn sql_types() -> Vec<(&'static str, &'static str)>;
    async fn post_create_table(pool: &PgPool);
}

pub trait DBObject {
    async fn insert(&self, pool: &PgPool, strategy: InsertStrategy, ignore: bool);

    #[allow(dead_code)]
    fn type_name() -> &'static str;
}
