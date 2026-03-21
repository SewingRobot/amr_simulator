use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<ServiceError> for tonic::Status {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound(msg) => tonic::Status::not_found(msg),
            ServiceError::Validation(msg) => tonic::Status::invalid_argument(msg),
            ServiceError::StorageError(msg) => tonic::Status::internal(msg),
            ServiceError::ProcessingError(msg) => tonic::Status::internal(msg),
            ServiceError::PathNotFound(msg) => tonic::Status::not_found(msg),
            ServiceError::Internal(msg) => tonic::Status::internal(msg),
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ServiceError::NotFound("Record not found".to_string()),
            other => ServiceError::Internal(other.to_string()),
        }
    }
}
