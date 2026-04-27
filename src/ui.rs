use std::{net::Ipv4Addr, path::PathBuf};

use iced::{Function, Subscription, Task, futures, keyboard};
use screen::{
    Screen,
    serverboot::{
        self, Console, DEFAULT_PORT, PORT_OFFSET, ServerCommunicationTwoWay, ServerTerminal,
        find_available_port,
    },
    servercreation,
    serverlist::{self, ServerList},
};

use crate::{
    core::{Game, SourceEngineVersion, get_arg_game_name},
    ui::{
        games::SOURCE_GAMES,
        screen::servercreation::download_server,
        server::{Server, Servers},
        themes::Theme,
    },
};
use futures::TryFutureExt;

pub mod components;
pub mod games;
pub mod screen;
pub mod server;
pub mod themes;

const SERVER_LIST_FILE_NAME: &str = "server_list.toml";

type Element<'a, Message> = iced::Element<'a, Message, Theme>;

pub struct State {
    screen: Screen,
    servers: Servers,
}

#[derive(Debug, Clone)]
pub enum Message {
    ServersLoaded(Result<Servers, screen::serverlist::Error>),
    UpdateServer(usize, Update),
    ServerCommunication(
        usize,
        Result<ServerCommunicationTwoWay, screen::serverboot::Error>,
    ),
    ServerList(serverlist::Message),
    ServerCreation(servercreation::Message),
    ServerTerminal(usize, serverboot::Message),
}

#[derive(Debug, Clone)]
pub enum Update {
    Downloading(f32),
    Finished(Result<(), servercreation::Error>),
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::Loading,
                servers: Servers::new(),
            },
            Task::perform(
                async {
                    futures::future::ready(get_config_path())
                        .and_then(|path| async move { Servers::fetch(path.as_path()).await })
                        .await
                },
                Message::ServersLoaded,
            ),
        )
    }

    pub fn title(&self) -> String {
        match self.screen {
            Screen::Loading | Screen::ServerList => "MANNager".into(),
            Screen::ServerCreation(_) => "MANNager - Creating a server".into(),
            Screen::ServerTerminal(id) => self
                .servers
                .get(id)
                .map(|Server { info, .. }| format!("MANNager - Server Terminal [{}]", info.name))
                .unwrap_or_else(|| "MANNager".to_string()),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServersLoaded(servers) => {
                self.servers = servers.unwrap_or_default();

                self.screen = Screen::ServerList;

                Task::none()
            }
            Message::UpdateServer(id, update) => {
                let Some(Server {
                    updating_percent: percent,
                    ..
                }) = self.servers.get_mut(id)
                else {
                    return Task::none();
                };

                *percent = match update {
                    Update::Downloading(_percent) => Some(_percent),
                    Update::Finished(_) => None,
                };

                Task::none()
            }
            Message::ServerCommunication(id, msg) => {
                let Ok(communication) = msg else {
                    return Task::none();
                };

                let Some(Server {
                    console: Some(console),
                    ..
                }) = self.servers.get_mut(id)
                else {
                    return Task::none();
                };

                match communication {
                    ServerCommunicationTwoWay::Input(sender) => {
                        console.sender = Some(sender);
                    }
                    ServerCommunicationTwoWay::Output(text) => {
                        console.output.push(text);
                    }
                }

                Task::none()
            }
            Message::ServerList(message) => {
                use serverlist::Action;

                let action = ServerList::update(&mut self.servers, message);

                match action {
                    Action::None => Task::none(),
                    Action::SaveServers => {
                        let servers = self.servers.clone();

                        Task::future(async {
                            futures::future::ready(get_config_path())
                                .and_then(|path| async move { servers.save(&path).await })
                                .await
                        })
                        .discard()
                    }
                    Action::CreateServer => {
                        self.screen = Screen::ServerCreation(servercreation::State::new());

                        Task::none()
                    }
                    Action::UpdateServer(id) => {
                        let Some(Server {
                            info,
                            updating_percent,
                            ..
                        }) = self.servers.get_mut(id)
                        else {
                            return Task::none();
                        };

                        *updating_percent = Some(0.0);

                        let server_path = info.path.clone();
                        let source_game = info.game.clone();

                        Task::sip(
                            download_server(server_path, source_game),
                            Update::Downloading,
                            Update::Finished,
                        )
                        .map(Message::UpdateServer.with(id))
                    }
                    Action::EditServer(id) => {
                        let Some(server) = self.servers.get_mut(id) else {
                            return Task::none();
                        };

                        server.is_editing = true;

                        Task::none()
                    }
                    Action::StopEditServer(id) => {
                        let Some(server) = self.servers.get_mut(id) else {
                            return Task::none();
                        };

                        server.is_editing = false;

                        Task::none()
                    }
                    Action::RunServer(id) => {
                        let Some(Server { info, console, .. }) = self.servers.get_mut(id) else {
                            return Task::none();
                        };

                        let Some(game_info) = SOURCE_GAMES
                            .iter()
                            .find(|game_info| game_info.game == info.game)
                        else {
                            return Task::none();
                        };

                        let binary_path = {
                            let server_path = &info.path;
                            let executable_path = &game_info.executable_path;

                            server_path.join(executable_path)
                        };

                        let port = info.port.unwrap_or_else(|| {
                            find_available_port(Ipv4Addr::new(0, 0, 0, 0), DEFAULT_PORT)
                        });

                        let args = {
                            let mut args = match game_info.engine {
                                SourceEngineVersion::Source1 => {
                                    format!("-console -game {}", get_arg_game_name(&info.game))
                                }
                                SourceEngineVersion::Source2 => "-dedicated".to_string(),
                            };

                            args.push_str(&format!(
                                " +hostname \"{}\" +map {} +maxplayers {} -nohltv -strictportbind +ip 0.0.0.0 -port {} -clientport {}",
                                info.name,
                                info.map,
                                info.max_players,
                                port,
                                port + PORT_OFFSET,
                            ));

                            if info.max_players > 32 && info.game == Game::TeamFortress2 {
                                args.push_str(" -unrestricted_maxplayers");
                            }

                            if let Some(token) = &info.gslt {
                                args.push_str(&format!(" +sv_setsteamaccount {token}"));
                            }

                            args
                        };

                        let (task, handle) = Task::run(
                            Console::start(binary_path, args, info.name.clone(), port),
                            Message::ServerCommunication.with(id),
                        )
                        .abortable();

                        *console = Some(Console::from_handle(handle, port));

                        self.screen = Screen::ServerTerminal(id);

                        task
                    }
                    Action::OpenTerminal(id) => {
                        self.screen = Screen::ServerTerminal(id);

                        Task::none()
                    }
                    Action::StopServer(id) => {
                        let Some(Server { console, .. }) = self.servers.get_mut(id) else {
                            return Task::none();
                        };

                        *console = None;

                        self.screen = Screen::ServerList;

                        Task::none()
                    }
                    Action::Run(task) => task.map(Message::ServerList),
                }
            }
            Message::ServerCreation(msg) => {
                use servercreation::Action;

                let Screen::ServerCreation(creation) = &mut self.screen else {
                    return Task::none();
                };

                match creation.update(msg) {
                    Action::None => Task::none(),
                    Action::SwitchToServerList => {
                        self.screen = Screen::ServerList;

                        Task::none()
                    }
                    Action::ServerCreated(server) => {
                        self.servers.push(Server::with_info(server));

                        self.screen = Screen::ServerList;

                        let servers = self.servers.clone();

                        Task::future(async {
                            futures::future::ready(get_config_path())
                                .and_then(|path| async move { servers.save(&path).await })
                                .await
                        })
                        .discard()
                    }
                    Action::Run(task) => task.map(Message::ServerCreation),
                }
            }
            Message::ServerTerminal(id, message) => {
                use serverboot::Action;

                let Some(Server {
                    console: Some(console),
                    ..
                }) = self.servers.get_mut(id)
                else {
                    return Task::none();
                };

                let action = ServerTerminal::update(console, message);

                match action {
                    Action::None => Task::none(),
                    Action::GoBack => {
                        self.screen = Screen::ServerList;

                        Task::none()
                    }
                    Action::Run(task) => task.map(Message::ServerTerminal.with(id)),
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Loading => screen::loading::loading(),
            Screen::ServerList => ServerList::view(&self.servers).map(Message::ServerList),
            Screen::ServerCreation(creation) => creation.view().map(Message::ServerCreation),
            Screen::ServerTerminal(index) => {
                let index = index.clone();

                // TODO: remove the unwrap
                let Server { info, console, .. } = &self.servers[index];

                ServerTerminal::view(&info.name, console.as_ref().unwrap())
                    .map(move |msg| Message::ServerTerminal(index, msg))
            }
        }
    }
}

fn get_config_path() -> Result<PathBuf, screen::serverlist::Error> {
    if let Ok(executable_directory) = std::env::current_dir() {
        let config_path = executable_directory.join(SERVER_LIST_FILE_NAME);

        if config_path.try_exists().unwrap_or(false) {
            return Ok(config_path);
        }
    }

    let project_path = directories::ProjectDirs::from("", "MANNager", "mannager-source")
        .ok_or(screen::serverlist::Error::NoServerListFile)?;

    let config_path = project_path.config_dir().join(SERVER_LIST_FILE_NAME);

    if config_path.try_exists().unwrap_or(false) {
        Ok(config_path)
    } else {
        Err(screen::serverlist::Error::NoServerListFile)
    }
}
