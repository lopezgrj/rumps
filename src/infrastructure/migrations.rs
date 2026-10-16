use sqlx::{AnyPool, Executor};

use crate::application::DatabaseKind;

const SQLITE_SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS rumps_schema_migrations (
        version BIGINT PRIMARY KEY,
        description TEXT NOT NULL,
        applied_at TEXT NOT NULL
    );
";

const POSTGRES_SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS rumps_schema_migrations (
        version BIGINT PRIMARY KEY,
        description TEXT NOT NULL,
        applied_at TEXT NOT NULL
    );
";

pub async fn migrate(pool: &AnyPool, kind: DatabaseKind) -> Result<(), sqlx::Error> {
    let schema = match kind {
        DatabaseKind::Sqlite => SQLITE_SCHEMA,
        DatabaseKind::Postgres => POSTGRES_SCHEMA,
    };

    pool.execute(schema).await?;

    let applied = sqlx::query(
        "INSERT INTO rumps_schema_migrations (version, description, applied_at)
         VALUES (1, 'Initial API schema version table', CURRENT_TIMESTAMP)
         ON CONFLICT (version) DO NOTHING",
    )
    .execute(pool)
    .await?;

    if applied.rows_affected() > 1 {
        return Err(sqlx::Error::Protocol(
            "schema migration inserted an unexpected number of rows".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migration_initializes_sqlite_schema() -> Result<(), Box<dyn std::error::Error>> {
        sqlx::any::install_default_drivers();
        let pool = sqlx::any::AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;

        migrate(&pool, DatabaseKind::Sqlite).await?;
        migrate(&pool, DatabaseKind::Sqlite).await?;

        let row: (i64, String) = sqlx::query_as(
            "SELECT version, description FROM rumps_schema_migrations WHERE version = 1",
        )
        .fetch_one(&pool)
        .await?;

        assert_eq!(row.0, 1);
        assert_eq!(row.1, "Initial API schema version table");
        Ok(())
    }
}
