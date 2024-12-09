use thiserror::{self};

#[derive(Debug, thiserror::Error)]
pub enum GenerateFilesError {
    #[error("Output directory {0} exists but overwrite_if_exists = false")]
    DirectoryExists(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
