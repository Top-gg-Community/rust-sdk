use thiserror::Error;

#[derive(Error, Debug)]
pub enum SDKError {
    #[error("Rate limited (Retry after: {retry_after:?})")]
    Ratelimited {
        retry_after: u64,
    },
}
