use std::{io, sync::Arc};

use thiserror::Error;
use zip::result::ZipError;

pub mod depotdownloader;
pub mod metamod;
pub mod portforwarder;
pub mod sourcemod;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceEngineVersion {
    Source1,
    Source2,
}

impl From<SourceEngineVersion> for u32 {
    fn from(value: SourceEngineVersion) -> Self {
        match value {
            SourceEngineVersion::Source1 => 1,
            SourceEngineVersion::Source2 => 2,
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Failed to create directories: {0}")]
    DirectoryCreationError(String),

    #[error("download request failed: {0}")]
    DownloadRequestError(Arc<reqwest::Error>),

    #[error("extraction error: {0}")]
    ArchiveExtractionError(Arc<ExtractError>),

    #[error("Failed to spawn the process: {0}")]
    SpawnProcessError(String),

    #[error("Failed to retrieve the latest version")]
    UnableToFindLatestVersionError,

    #[error("io failed: {0}")]
    Io(Arc<io::Error>),
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("")]
    ZipError(#[from] ZipError),

    #[error(transparent)]
    TarError(#[from] io::Error),
}

impl From<ExtractError> for Error {
    fn from(error: ExtractError) -> Self {
        Self::ArchiveExtractionError(Arc::new(error))
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(Arc::new(error))
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::DownloadRequestError(Arc::new(error))
    }
}
