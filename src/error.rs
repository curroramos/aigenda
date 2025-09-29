use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("json: {0}")] Json(#[from] serde_json::Error),
    #[error("parse date: {0}")] ChronoParse(#[from] chrono::ParseError),
    #[error("{0}")] Other(String),
}

pub type AppResult<T> = Result<T, AppError>;
