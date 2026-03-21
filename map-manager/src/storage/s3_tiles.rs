use aws_sdk_s3::Client as S3Client;
use uuid::Uuid;

use crate::error::ServiceError;

/// Potree tile storage on MinIO (S3-compatible).
///
/// Tiles are stored at: `s3://{bucket}/tiles/{map_id}/{node_name}`
/// Raw uploads are stored at: `s3://{bucket}/raw/{map_id}/{filename}`
#[allow(dead_code)]
pub struct TileStorage {
    client: S3Client,
    bucket: String,
}

#[allow(dead_code)]
impl TileStorage {
    pub fn new(client: S3Client, bucket: String) -> Self {
        Self { client, bucket }
    }

    /// Upload a Potree tile to MinIO.
    ///
    /// Returns the S3 key of the uploaded tile.
    pub async fn upload_tile(
        &self,
        map_id: Uuid,
        node_name: &str,
        data: Vec<u8>,
    ) -> Result<String, ServiceError> {
        let key = format!("tiles/{}/{}", map_id, node_name);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(data.into())
            .content_type("application/octet-stream")
            .send()
            .await
            .map_err(|e| ServiceError::StorageError(format!("Failed to upload tile: {}", e)))?;

        Ok(key)
    }

    /// Get a Potree tile from MinIO.
    ///
    /// `node_name` is the Potree node identifier (e.g., "r", "r0", "r01").
    /// `lod` is currently unused but reserved for future LOD-based path resolution.
    pub async fn get_tile(
        &self,
        map_id: Uuid,
        node_name: &str,
        _lod: u32,
    ) -> Result<Vec<u8>, ServiceError> {
        let key = format!("tiles/{}/{}", map_id, node_name);

        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| ServiceError::StorageError(format!("Failed to get tile: {}", e)))?;

        let data = resp
            .body
            .collect()
            .await
            .map_err(|e| ServiceError::StorageError(format!("Failed to read tile body: {}", e)))?
            .into_bytes()
            .to_vec();

        Ok(data)
    }

    /// Delete all tiles for a given map.
    pub async fn delete_tiles(&self, map_id: Uuid) -> Result<(), ServiceError> {
        let prefix = format!("tiles/{}/", map_id);

        // List and delete all objects with the prefix
        let list_resp = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .send()
            .await
            .map_err(|e| {
                ServiceError::StorageError(format!("Failed to list tiles for deletion: {}", e))
            })?;

        for obj in list_resp.contents() {
            if let Some(key) = obj.key() {
                self.client
                    .delete_object()
                    .bucket(&self.bucket)
                    .key(key)
                    .send()
                    .await
                    .map_err(|e| {
                        ServiceError::StorageError(format!("Failed to delete tile: {}", e))
                    })?;
            }
        }

        Ok(())
    }
}
