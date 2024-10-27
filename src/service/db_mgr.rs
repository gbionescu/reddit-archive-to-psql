use crate::service::DBTable;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use sqlx::Row;

pub struct DBManager {
    pub pool: PgPool,
}

#[derive(PartialEq)]
pub enum InsertStrategy {
    InsertIgnore,
    InsertUpdate,
}

impl DBManager {
    // Create a new DBManager instance.
    pub async fn new(host: &str, port: u16, user: &str, password: &str, db_name: &str) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(200)
            .acquire_timeout(std::time::Duration::from_secs(300))
            .acquire_slow_threshold(std::time::Duration::from_secs(300))
            .connect(&format!(
                "postgres://{}:{}@{}:{}/{}",
                user, password, host, port, db_name
            ))
            .await
            .expect("Failed to connect to Postgres.");

        DBManager { pool }
    }

    // Check if a table exists.
    async fn table_exists(&self, table_name: &str) -> bool {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1")
                .bind(table_name)
                .fetch_one(&self.pool)
                .await
                .expect("Failed to fetch row.");

        row.0 == 1
    }

    // Create a table given a table name and a list of column names and types.
    async fn create_table(&self, table_name: &str, sql_types: &Vec<(&str, &str)>) {
        let mut sql = format!("CREATE TABLE {} (", table_name);

        for (i, (name, sql_type)) in sql_types.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }

            sql.push_str(name);
            sql.push_str(" ");
            sql.push_str(sql_type);
        }

        sql.push_str(");");

        sqlx::query(&sql)
            .execute(&self.pool)
            .await
            .expect("Failed to create table.");
    }

    // Get the schema of a table.
    async fn get_schema(&self, table_name: &str) -> Vec<(String, String)> {
        let schema = sqlx::query(
            "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = $1",
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .expect("Failed to fetch schema.");

        schema
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let data_type: String = row.get(1);

                (name, data_type)
            })
            .collect()
    }

    // Get the number of rows in a table.
    pub async fn get_table_count<T: DBTable>(&self) -> i64 {
        let table_name = T::table_name();
        let row: (i64,) = sqlx::query_as(&format!(
            "SELECT reltuples::BIGINT AS estimate FROM pg_class WHERE relname = '{}'",
            table_name
        ))
        .fetch_one(&self.pool)
        .await
        .expect("Failed to fetch row.");

        row.0
    }

    // Get the size of a table in GB.
    pub async fn get_table_size<T: DBTable>(&self) -> f64 {
        let table_name = T::table_name();
        let row: (i64,) =
            sqlx::query_as(&format!("SELECT pg_total_relation_size('{}')", table_name))
                .fetch_one(&self.pool)
                .await
                .expect("Failed to fetch row.");

        // Convert to GB and keep 3 decimal places.
        (row.0 as f64 / 1024.0 / 1024.0 / 1024.0 * 1000.0).round() / 1000.0
    }

    // Check if the table exists and create it if it doesn't.
    pub async fn check_tables<T: DBTable>(&self) {
        let sql_types_sub = T::sql_types();
        let target_table = T::table_name();

        log::info!("Checking table {}", target_table);
        if !self.table_exists(target_table).await {
            print!("Creating table {}", target_table);
            self.create_table(target_table, &sql_types_sub).await;
            T::post_create_table(&self.pool).await;
        } else {
            log::info!("Table {} exists.", target_table);
        }

        let schema_sub = self.get_schema(target_table).await;
        for (name, data_type) in schema_sub.clone() {
            // Find mismatched columns.
            let mut found = false;
            for (expected_name, expected_type) in &sql_types_sub {
                if name == *expected_name {
                    if data_type != *expected_type {
                        log::error!(
                            "Column {} has type {} but expected type {}.",
                            name,
                            data_type,
                            expected_type
                        );
                    }

                    found = true;
                    break;
                }
            }

            // If the column is not found in the expected schema, panic.
            if !found {
                log::error!("Column {} not found in expected schema.", name);
            }
        }

        // Check for extra columns.
        let mut found = false;
        for (name, _) in schema_sub.clone() {
            for (expected_name, _) in &sql_types_sub {
                if name == *expected_name {
                    found = true;
                    break;
                }
            }

            if !found {
                log::error!("Extra column {} found in table.", name);
            }
        }

        // Check for missing columns.
        for (expected_name, _) in &sql_types_sub {
            let mut found = false;
            for (name, _) in schema_sub.clone() {
                if name == *expected_name {
                    found = true;
                    break;
                }
            }

            if !found {
                log::error!("Column {} not found in table.", expected_name);
            }
        }
    }
}
