use std::{
    fs,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    str::FromStr,
};

use decoder::Value;

use crate::{
    core::Game,
    ui::screen::{serverboot::Console, serverlist::Error},
};

#[derive(Debug, Clone)]
pub struct Servers(pub Vec<Server>);

impl Servers {
    pub async fn fetch(path: &Path) -> Result<Self, Error> {
        match path.try_exists() {
            Ok(true) => {
                let file_contents = fs::read_to_string(path).unwrap();
                decoder::run(toml::from_str, Servers::decode, &file_contents)
                    .map_err(|_| Error::NoServerListFile)
            }
            _ => Err(Error::NoServerListFile),
        }
    }

    pub async fn save(&self, path: &Path) -> Result<(), Error> {
        let toml = toml::to_string_pretty(&self.encode()).map_err(|_| Error::ServerSaveError)?;

        tokio::fs::write(path, toml)
            .await
            .map_err(|_| Error::ServerSaveError)?;

        Ok(())
    }

    pub fn decode(value: Value) -> Result<Self, decoder::Error> {
        use decoder::decode::{map, sequence};

        let servers: Vec<ServerInfo> = map(value)?
            .optional("servers", sequence(ServerInfo::decode))?
            .unwrap_or_default();

        Ok(Servers(
            servers
                .into_iter()
                .map(|info| Server::with_info(info))
                .collect(),
        ))
    }

    pub fn encode(&self) -> Value {
        use decoder::encode::{map, sequence};

        let servers = self.iter().map(|server| &server.info);

        map([("servers", sequence(ServerInfo::encode, servers))]).into_value()
    }
}

impl Default for Servers {
    fn default() -> Self {
        Self(vec![])
    }
}

impl Deref for Servers {
    type Target = Vec<Server>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Servers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct Server {
    pub info: ServerInfo,
    pub console: Option<Console>,
    pub is_downloading_sourcemod: bool,
    pub updating_percent: Option<f32>,
    pub is_editing: bool,
}

impl Server {
    pub fn new() -> Self {
        Self {
            info: ServerInfo::default(),
            console: None,
            is_downloading_sourcemod: false,
            updating_percent: None,
            is_editing: false,
        }
    }

    pub fn with_info(info: ServerInfo) -> Self {
        Self {
            info,
            console: None,
            is_downloading_sourcemod: false,
            updating_percent: None,
            is_editing: false,
        }
    }

    pub fn is_running(&self) -> bool {
        self.console.is_some()
    }

    pub fn is_updating(&self) -> bool {
        self.updating_percent.is_some()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ServerInfo {
    pub name: String,
    pub game: Game,
    pub description: Option<String>,
    pub path: PathBuf,
    pub map: String,
    pub max_players: u32,
    pub password: Option<String>,
    pub port: Option<u16>,
    pub gslt: Option<String>,
}

impl ServerInfo {
    pub fn decode(value: Value) -> Result<Self, decoder::Error> {
        use decoder::decode::{map, string, u16, u32};

        let mut server = map(value)?;

        Ok(Self {
            name: server.required("name", string)?,
            game: server.required("game", Game::decode)?,
            description: server.optional("description", string)?,
            path: PathBuf::from_str(&server.required("path", string)?).expect("no bueno"),
            map: server.required("map", string)?,
            max_players: server.required("max_players", u32)?,
            password: server.optional("password", string)?,
            port: server.optional("port", u16)?,
            gslt: server.optional("gslt", string)?,
        })
    }

    pub fn encode(&self) -> Value {
        use decoder::encode::{map, optional, string, u16, u32};

        map([
            ("name", string(&self.name)),
            ("game", string(&self.game.to_string())),
            ("description", optional(string, self.description.clone())),
            ("path", string(self.path.to_str().unwrap_or_default())),
            ("map", string(&self.map)),
            ("max_players", u32(self.max_players)),
            ("password", optional(string, self.password.clone())),
            ("port", optional(u16, self.port)),
            ("gslt", optional(string, self.gslt.clone())),
        ])
        .into()
    }
}
