use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ServiceError;
use crate::models::map::MapRecord;

/// Map metadata repository backed by PostgreSQL.
#[allow(dead_code)]
pub struct MapRepository {
    pool: PgPool,
}

#[allow(dead_code)]
impl MapRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List all maps.
    pub async fn list(&self) -> Result<Vec<MapRecord>, ServiceError> {
        let records = sqlx::query_as::<_, MapRecord>(
            "SELECT id, name, version, metadata, created_at, updated_at FROM maps ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    /// Get a map by ID.
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<MapRecord>, ServiceError> {
        let record = sqlx::query_as::<_, MapRecord>(
            "SELECT id, name, version, metadata, created_at, updated_at FROM maps WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    /// Create a new map.
    pub async fn create(&self, name: &str) -> Result<MapRecord, ServiceError> {
        let record = sqlx::query_as::<_, MapRecord>(
            r#"
            INSERT INTO maps (name, metadata)
            VALUES ($1, '{}')
            RETURNING id, name, version, metadata, created_at, updated_at
            "#,
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    /// Delete a map by ID.
    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        let result = sqlx::query("DELETE FROM maps WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound(format!("Map {} not found", id)));
        }

        Ok(())
    }
}
