use std::io;

use preflate_bindings::PreflateError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use dump::dump;
pub use rebuild::rebuild;
mod dump;
mod rebuild;
mod shared;

#[derive(Error, Debug)]
pub enum Error {
    /// An IO error occurred
    #[error("io error")]
    IoError(#[from] io::Error),
    /// Error occured while running preflate
    #[error("preflate call failed")]
    PreflateError,
    /// Error while reading the zip archive
    #[error("zip archive error")]
    ZipError(#[from] zip::result::ZipError),
}

impl From<PreflateError> for Error {
    fn from(_: PreflateError) -> Self {
        Error::PreflateError
    }
}

#[derive(Serialize, Deserialize)]
pub struct RebuildInfo {
    /// Name of the original zip file
    pub original_filename: String,
    /// File containing the zip headers and central directory
    pub headers: String,
    /// List of all files that should be reinserted
    pub files: Vec<ReinsertInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ReinsertInfo {
    /// Offset at which the data should be inserted
    pub offset: u64,
    /// File containing insertable data
    pub data: String,
    /// If present, file containing the preflate diff. If not present, data is reinserted as-is
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    /// If present, BLAKE3 hash of the compressed file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}
