use sqlx::PgPool;

use crate::storage::s3::S3Storage;

/// AssetService gRPC implementation.
///
/// This struct holds shared state for the gRPC service handlers.
/// When proto definitions are finalized, this will implement the
/// generated AssetService trait.
///
/// # Planned RPCs
/// - `ListAssets` — list assets with filtering by type, tags, name
/// - `GetAsset` — retrieve asset metadata and file info by ID
/// - `CreateAsset` — chunked streaming upload of asset file + metadata
/// - `DeleteAsset` — cascade delete: DB record + MinIO objects
/// - `DownloadAsset` — chunked streaming download from MinIO
/// - `ValidateAsset` — run validation checks on an uploaded asset
#[derive(Debug, Clone)]
pub struct AssetServiceImpl {
    pub pool: PgPool,
    pub s3: S3Storage,
}

impl AssetServiceImpl {
    pub fn new(pool: PgPool, s3: S3Storage) -> Self {
        Self { pool, s3 }
    }
}

// TODO: Implement the generated AssetService trait once proto files are compiled.
// Each RPC will initially return tonic::Status::unimplemented().
//
// Example:
//
// #[tonic::async_trait]
// impl asset_service_server::AssetService for AssetServiceImpl {
//     async fn list_assets(&self, request: Request<ListAssetsRequest>)
//         -> Result<Response<ListAssetsResponse>, Status>
//     {
//         Err(Status::unimplemented("list_assets not yet implemented"))
//     }
//
//     async fn get_asset(&self, request: Request<GetAssetRequest>)
//         -> Result<Response<GetAssetResponse>, Status>
//     {
//         Err(Status::unimplemented("get_asset not yet implemented"))
//     }
//
//     async fn create_asset(&self, request: Request<Streaming<CreateAssetRequest>>)
//         -> Result<Response<CreateAssetResponse>, Status>
//     {
//         Err(Status::unimplemented("create_asset not yet implemented"))
//     }
//
//     async fn delete_asset(&self, request: Request<DeleteAssetRequest>)
//         -> Result<Response<DeleteAssetResponse>, Status>
//     {
//         Err(Status::unimplemented("delete_asset not yet implemented"))
//     }
//
//     type DownloadAssetStream = ...;
//     async fn download_asset(&self, request: Request<DownloadAssetRequest>)
//         -> Result<Response<Self::DownloadAssetStream>, Status>
//     {
//         Err(Status::unimplemented("download_asset not yet implemented"))
//     }
//
//     async fn validate_asset(&self, request: Request<ValidateAssetRequest>)
//         -> Result<Response<ValidateAssetResponse>, Status>
//     {
//         Err(Status::unimplemented("validate_asset not yet implemented"))
//     }
// }
