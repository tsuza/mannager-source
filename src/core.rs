use std::{io, sync::Arc};

use decoder::Value;
use snafu::prelude::*;
use zip::result::ZipError;

pub mod depotdownloader;
pub mod metamod;
pub mod portforwarder;
pub mod sourcemod;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Copy)]
pub enum Game {
    #[default]
    TeamFortress2,
    CounterStrikeGlobalOffensive,
    CounterStrikeSource,
    LeftForDead1,
    LeftForDead2,
    HalfLife2DM,
    NoMoreRoomInHell,
    CounterStrike2,
    Deadlock,
}

impl Game {
    pub fn decode(value: Value) -> Result<Self, decoder::Error> {
        use decoder::decode::string;

        let game = string(value)?;

        game.parse().map_err(|str| decoder::Error::Custom(str))
    }

    pub fn encode(&self) -> Value {
        use decoder::encode::string;

        string(self.to_string())
    }
}

impl std::str::FromStr for Game {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Team Fortress 2" => Ok(Game::TeamFortress2),
            "Counter-Strike: Source" => Ok(Game::CounterStrikeSource),
            "Counter-Strike: Global Offensive" => Ok(Game::CounterStrikeGlobalOffensive),
            "Left For Dead 1" => Ok(Game::LeftForDead1),
            "Left For Dead 2" => Ok(Game::LeftForDead2),
            "Half-Life 2: DM" => Ok(Game::HalfLife2DM),
            "No More Room In Hell" => Ok(Game::NoMoreRoomInHell),
            "Counter-Strike: 2" => Ok(Game::CounterStrike2),
            "Deadlock" => Ok(Game::Deadlock),
            _ => Err(format!("'{s}' is not a valid game")),
        }
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Game::TeamFortress2 => write!(f, "Team Fortress 2"),
            Game::CounterStrikeSource => write!(f, "Counter-Strike: Source"),
            Game::CounterStrikeGlobalOffensive => write!(f, "Counter-Strike: Global Offensive"),
            Game::LeftForDead1 => write!(f, "Left For Dead 1"),
            Game::LeftForDead2 => write!(f, "Left For Dead 2"),
            Game::HalfLife2DM => write!(f, "Half Life 2: DM"),
            Game::NoMoreRoomInHell => write!(f, "No More Room In Hell"),
            Game::CounterStrike2 => write!(f, "Counter-Strike: 2"),
            Game::Deadlock => write!(f, "Deadlock"),
        }
    }
}

impl From<Game> for u32 {
    fn from(value: Game) -> Self {
        match value {
            Game::TeamFortress2 => 232250,
            Game::CounterStrikeSource => 232330,
            Game::CounterStrike2 => 730,
            Game::CounterStrikeGlobalOffensive => 740,
            Game::LeftForDead1 => 222840,
            Game::LeftForDead2 => 222860,
            Game::HalfLife2DM => 232370,
            Game::NoMoreRoomInHell => 317670,
            Game::Deadlock => 1422450,
        }
    }
}

pub fn get_arg_game_name(game: &Game) -> &'static str {
    match game {
        Game::TeamFortress2 => "tf",
        Game::CounterStrikeSource => "cstrike",
        Game::CounterStrikeGlobalOffensive => "csgo",
        Game::CounterStrike2 => "cs",
        Game::LeftForDead1 => "left4dead",
        Game::LeftForDead2 => "left4dead2",
        Game::HalfLife2DM => "hl2mp",
        Game::NoMoreRoomInHell => "nmrih",
        Game::Deadlock => "deadlock",
    }
}

#[derive(Snafu, Debug, Clone)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to create directories: {source}"))]
    DirectoryCreationError {
        #[snafu(source(from(io::Error, Arc::new)))]
        source: Arc<io::Error>,
    },

    #[snafu(display("download request failed: {source}"))]
    DownloadRequestError {
        #[snafu(source(from(reqwest::Error, Arc::new)))]
        source: Arc<reqwest::Error>,
    },

    #[snafu(display("extraction error: {source}"))]
    ArchiveExtractionError {
        #[snafu(source(from(ExtractError, Arc::new)))]
        source: Arc<ExtractError>,
    },

    #[snafu(display("Failed to spawn the process: {source}"))]
    SpawnProcessError {
        #[snafu(source(from(io::Error, Arc::new)))]
        source: Arc<io::Error>,
    },

    #[snafu(display("Failed to retrieve the latest version"))]
    UnableToFindLatestVersionError,

    #[snafu(display("io failed: {source}"))]
    Io {
        #[snafu(source(from(io::Error, Arc::new)))]
        source: Arc<io::Error>,
    },
}

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum ExtractError {
    #[snafu(display("{source}"))]
    ZipError { source: ZipError },

    #[snafu(display("{source}"))]
    TarError { source: io::Error },
}
