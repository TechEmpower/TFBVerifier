use std::{env, io, num};

use thiserror::Error;

pub type VerifierResult<T> = Result<T, VerifierError>;

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("Curl error occurred")]
    CurlError(#[from] curl::Error),

    #[error("IO error occurred")]
    IoError(#[from] io::Error),

    #[error("Serde json error")]
    SerdeJsonError(#[from] serde_json::error::Error),

    #[error("Environment variable error occurred")]
    EnvVarError(#[from] env::VarError),

    #[error("Parse int error occurred")]
    ParseIntError(#[from] num::ParseIntError),

    #[error("Strum parse error occurred")]
    StrumParseError(#[from] strum::ParseError),

    #[error("Invalid test type error: {0}")]
    InvalidTestType(String),

    #[error("Invalid database type error: {0}")]
    InvalidDatabaseType(String),
}
