use thiserror::Error;


#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Commit retry count exceeded!")]
    CommitRetryCountExceeded,
    #[error("Database error! {0:?}")]
    DatabaseError(#[from] diesel::result::Error),
}
