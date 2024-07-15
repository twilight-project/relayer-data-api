use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Commit retry count exceeded!")]
    CommitRetryCountExceeded,
    #[error("Database error! {0:?}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Crossbeam error {0:?}")]
    CrossbeamChannel(String),
    #[error("Json serde error {0:?}")]
    JsonError(#[from] serde_json::Error),
    #[error("Connection pool error {0:?}")]
    R2d2(#[from] r2d2::Error),
    #[error("Redis error {0:?}")]
    Redis(#[from] redis::RedisError),
}
