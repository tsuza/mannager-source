use std::io;

use thiserror::Error;
use zip::result::ZipError;

pub mod depotdownloader;
pub mod metamod;
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to create directories: {0}")]
    DirectoryCreationError(String),

    #[error(transparent)]
    DownloadRequestError(#[from] reqwest::Error),

    #[error(transparent)]
    ArchiveExtractionError(#[from] ExtractError),

    #[error("Failed to spawn the process: {0}")]
    SpawnProcessError(String),

    #[error("Failed to retrieve the latest version")]
    UnableToFindLatestVersionError,
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("")]
    ZipError(#[from] ZipError),

    #[error(transparent)]
    TarError(#[from] io::Error),
}
