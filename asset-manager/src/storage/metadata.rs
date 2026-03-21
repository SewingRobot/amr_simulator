use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ServiceError;
use crate::models::asset::{AssetFileRecord, AssetRecord};

/// Filter criteria for listing assets.
#[derive(Debug, Default)]
pub struct AssetFilter {
    pub asset_type: Option<String>,
    pub name_contains: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Repository for asset metadata CRUD operations backed by PostgreSQL.
#[derive(Debug, Clone)]
pub struct AssetRepository {
    pool: PgPool,
}

impl AssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// List assets matching the given filter criteria.
    pub async fn list(&self, filter: &AssetFilter) -> Result<Vec<AssetRecord>, ServiceError> {
        let mut query = String::from("SELECT * FROM assets WHERE 1=1");
        let mut args: Vec<String> = Vec::new();

        if let Some(ref asset_type) = filter.asset_type {
            args.push(asset_type.clone());
            query.push_str(&format!(" AND asset_type = ${}", args.len()));
        }

        if let Some(ref name) = filter.name_contains {
            args.push(format!("%{name}%"));
            query.push_str(&format!(" AND name ILIKE ${}", args.len()));
        }

        query.push_str(" ORDER BY created_at DESC");

        // Build the query dynamically with sqlx
        let mut sqlx_query = sqlx::query_as::<_, AssetRecord>(&query);
        for arg in &args {
            sqlx_query = sqlx_query.bind(arg);
        }

        let records = sqlx_query
            .fetch_all(&self.pool)
            .await
            .map_err(ServiceError::from)?;

        Ok(records)
    }

    /// Get a single asset by its ID.
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<AssetRecord>, ServiceError> {
        let record = sqlx::query_as::<_, AssetRecord>("SELECT * FROM assets WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(ServiceError::from)?;

        Ok(record)
    }

    /// Create a new asset record.
    pub async fn create(&self, asset: &AssetRecord) -> Result<AssetRecord, ServiceError> {
        let record = sqlx::query_as::<_, AssetRecord>(
            r#"INSERT INTO assets (id, asset_type, name, description, properties, tags, version)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(asset.id)
        .bind(&asset.asset_type)
        .bind(&asset.name)
        .bind(&asset.description)
        .bind(&asset.properties)
        .bind(&asset.tags)
        .bind(asset.version)
        .fetch_one(&self.pool)
        .await
        .map_err(ServiceError::from)?;

        Ok(record)
    }

    /// Delete an asset by its ID. Associated files are cascade-deleted.
    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        let result = sqlx::query("DELETE FROM assets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(ServiceError::from)?;

        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound(format!("Asset {id} not found")));
        }

        Ok(())
    }

    /// Create a new asset file record.
    pub async fn create_file(
        &self,
        file: &AssetFileRecord,
    ) -> Result<AssetFileRecord, ServiceError> {
        let record = sqlx::query_as::<_, AssetFileRecord>(
            r#"INSERT INTO asset_files (id, asset_id, format, storage_path, size_bytes, checksum_sha256)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(file.id)
        .bind(file.asset_id)
        .bind(&file.format)
        .bind(&file.storage_path)
        .bind(file.size_bytes)
        .bind(&file.checksum_sha256)
        .fetch_one(&self.pool)
        .await
        .map_err(ServiceError::from)?;

        Ok(record)
    }

    /// List all files associated with an asset.
    pub async fn list_files_by_asset(
        &self,
        asset_id: Uuid,
    ) -> Result<Vec<AssetFileRecord>, ServiceError> {
        let records = sqlx::query_as::<_, AssetFileRecord>(
            "SELECT * FROM asset_files WHERE asset_id = $1 ORDER BY created_at",
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(ServiceError::from)?;

        Ok(records)
    }
}
