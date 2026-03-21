use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;

use crate::error::ServiceError;

/// S3-compatible storage client for MinIO/S3 operations.
#[derive(Debug, Clone)]
pub struct S3Storage {
    client: Client,
}

impl S3Storage {
    /// Create a new S3Storage from an existing AWS S3 client.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Upload a file to the specified bucket and key.
    /// Returns the ETag of the uploaded object.
    pub async fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
    ) -> Result<String, ServiceError> {
        let body = ByteStream::from(data);

        let result = self
            .client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(body)
            .send()
            .await
            .map_err(|e| ServiceError::StorageError(format!("S3 upload failed: {e}")))?;

        let etag = result
            .e_tag()
            .unwrap_or("unknown")
            .to_string();

        tracing::debug!("Uploaded s3://{bucket}/{key} (etag: {etag})");
        Ok(etag)
    }

    /// Download a file from the specified bucket and key.
    /// Returns the file contents as a byte vector.
    pub async fn download_file(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<Vec<u8>, ServiceError> {
        let result = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| ServiceError::StorageError(format!("S3 download failed: {e}")))?;

        let data = result
            .body
            .collect()
            .await
            .map_err(|e| ServiceError::StorageError(format!("S3 read body failed: {e}")))?
            .into_bytes()
            .to_vec();

        tracing::debug!("Downloaded s3://{bucket}/{key} ({} bytes)", data.len());
        Ok(data)
    }

    /// Delete a file from the specified bucket and key.
    pub async fn delete_file(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(), ServiceError> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| ServiceError::StorageError(format!("S3 delete failed: {e}")))?;

        tracing::debug!("Deleted s3://{bucket}/{key}");
        Ok(())
    }
}
