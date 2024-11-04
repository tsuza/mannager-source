use std::{io, sync::Arc};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zip::result::ZipError;

pub mod depotdownloader;
pub mod metamod;
pub mod portforwarder;
pub mod sourcemod;

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum SourceEngineVersion {
    #[default]
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

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum SourceAppIDs {
    #[default]
    TeamFortress2,
    CounterStrikeSource,
    CounterStrike2,
    LeftForDead1,
    LeftForDead2,
    HalfLife2DM,
    NoMoreRoomInHell,
}

impl From<SourceAppIDs> for u32 {
    fn from(value: SourceAppIDs) -> Self {
        match value {
            SourceAppIDs::TeamFortress2 => 232250,
            SourceAppIDs::CounterStrikeSource => 232330,
            SourceAppIDs::CounterStrike2 => 730,
            SourceAppIDs::LeftForDead1 => 222840,
            SourceAppIDs::LeftForDead2 => 222860,
            SourceAppIDs::HalfLife2DM => 232370,
            SourceAppIDs::NoMoreRoomInHell => 317670,
        }
    }
}

pub fn get_arg_game_name(game: &SourceAppIDs) -> &'static str {
    match game {
        SourceAppIDs::TeamFortress2 => "tf",
        SourceAppIDs::CounterStrikeSource => "cstrike",
        SourceAppIDs::CounterStrike2 => "cs",
        SourceAppIDs::LeftForDead1 => "left4dead",
        SourceAppIDs::LeftForDead2 => "left4dead2",
        SourceAppIDs::HalfLife2DM => "hl2mp",
        SourceAppIDs::NoMoreRoomInHell => "nmrih",
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
