pub mod models;

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Create a PostgreSQL connection pool.
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("Failed to connect to database")?;

    tracing::info!("Database connection pool established");
    Ok(pool)
}

/// Run SQL migration files from the migrations/ directory.
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    tracing::info!("Running database migrations...");

    let migrations = [
        include_str!("../../migrations/001_create_users.sql"),
        include_str!("../../migrations/002_create_robots.sql"),
        include_str!("../../migrations/003_create_maps.sql"),
        include_str!("../../migrations/004_create_missions.sql"),
        include_str!("../../migrations/005_create_roadmap.sql"),
        include_str!("../../migrations/006_seed_roadmap.sql"),
    ];

    for (i, migration) in migrations.iter().enumerate() {
        sqlx::raw_sql(migration)
            .execute(pool)
            .await
            .with_context(|| format!("Failed to run migration {}", i + 1))?;
        tracing::info!("Migration {} applied successfully", i + 1);
    }

    tracing::info!("All migrations completed");
    Ok(())
}
