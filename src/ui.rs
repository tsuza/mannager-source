use std::{net::Ipv4Addr, path::PathBuf};

use iced::{Function, Subscription, Task, clipboard, futures, keyboard};
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
    core::{Game, get_arg_game_name},
    ui::{
        components::selectable_text,
        screen::servercreation::download_server,
        server::{Server, Servers},
        themes::Theme,
    },
};
use futures::TryFutureExt;

pub mod components;
pub mod games;
pub mod icons;
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
    Copy,
    SelectedText(Vec<(f32, String)>),
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
                servers: Servers(vec![]),
            },
            Task::perform(
                async {
                    get_config_path()
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

    pub fn subscription(&self) -> Subscription<Message> {
        fn handle_hotkey(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
            match key.as_ref() {
                keyboard::Key::Character("c") if modifiers.command() => Some(Message::Copy),
                _ => None,
            }
        }

        keyboard::on_key_press(handle_hotkey)
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
            Message::Copy => selectable_text::selected(Message::SelectedText),
            Message::SelectedText(contents) => {
                let mut last_y = None;
                let contents = contents
                    .into_iter()
                    .fold(String::new(), |acc, (y, content)| {
                        if let Some(_y) = last_y {
                            let new_line = if y == _y { "" } else { "\n" };
                            last_y = Some(y);

                            format!("{acc}{new_line}{content}")
                        } else {
                            last_y = Some(y);

                            content
                        }
                    });

                if contents.is_empty() {
                    return Task::none();
                }

                clipboard::write(contents)
            }
            Message::ServerList(message) => {
                let action = ServerList::update(&mut self.servers, message);

                match action {
                    serverlist::Action::None => Task::none(),
                    serverlist::Action::SaveServers => {
                        let servers = self.servers.clone();

                        Task::future(async {
                            get_config_path()
                                .and_then(|path| async move { servers.save(&path).await })
                                .await
                        })
                        .discard()
                    }

                    serverlist::Action::CreateServer => {
                        self.screen = Screen::ServerCreation(servercreation::State::new());

                        Task::none()
                    }
                    serverlist::Action::UpdateServer(id) => {
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
                    serverlist::Action::EditServer(id) => {
                        let Some(server) = self.servers.get_mut(id) else {
                            return Task::none();
                        };

                        server.is_editing = true;

                        Task::none()
                    }
                    serverlist::Action::StopEditServer(id) => {
                        let Some(server) = self.servers.get_mut(id) else {
                            return Task::none();
                        };

                        server.is_editing = false;

                        Task::none()
                    }
                    serverlist::Action::RunServer(id) => {
                        let Some(Server { info, console, .. }) = self.servers.get_mut(id) else {
                            return Task::none();
                        };

                        #[cfg(target_os = "linux")]
                        const SRCDS_EXEC_NAME: &str = "srcds_run";

                        #[cfg(target_os = "windows")]
                        const SRCDS_EXEC_NAME: &str = "srcds-fix.exe";

                        let binary_path = info.path.join(SRCDS_EXEC_NAME);

                        let port = info.port.unwrap_or_else(|| {
                            find_available_port(Ipv4Addr::new(0, 0, 0, 0), DEFAULT_PORT)
                        });

                        let args = format!(
                            "-console -game {} +hostname \"{}\" +map {} +maxplayers {} -nohltv -strictportbind +ip 0.0.0.0 -port {} -clientport {}{}{}",
                            get_arg_game_name(&info.game),
                            info.name,
                            info.map,
                            info.max_players,
                            port,
                            port + PORT_OFFSET,
                            if info.max_players > 32 && info.game == Game::TeamFortress2 {
                                " -unrestricted_maxplayers"
                            } else {
                                ""
                            },
                            info.gslt.as_ref().map_or(String::new(), |token| format!(
                                "+sv_setsteamaccount {}",
                                token
                            ))
                        );

                        let (task, handle) = Task::run(
                            Console::start(binary_path, args, info.name.clone(), port),
                            Message::ServerCommunication.with(id),
                        )
                        .abortable();

                        *console = Some(Console::from_handle(handle, port));

                        self.screen = Screen::ServerTerminal(id);

                        task
                    }
                    serverlist::Action::OpenTerminal(id) => {
                        self.screen = Screen::ServerTerminal(id);

                        Task::none()
                    }
                    serverlist::Action::StopServer(id) => {
                        let Some(Server { console, .. }) = self.servers.get_mut(id) else {
                            return Task::none();
                        };

                        *console = None;

                        self.screen = Screen::ServerList;

                        Task::none()
                    }
                    serverlist::Action::Run(task) => task.map(Message::ServerList),
                }
            }
            Message::ServerCreation(msg) => {
                let Screen::ServerCreation(creation) = &mut self.screen else {
                    return Task::none();
                };

                match creation.update(msg) {
                    servercreation::Action::None => Task::none(),
                    servercreation::Action::SwitchToServerList => {
                        self.screen = Screen::ServerList;

                        Task::none()
                    }
                    servercreation::Action::ServerCreated(server) => {
                        self.servers.push(Server::with_info(server));

                        self.screen = Screen::ServerList;

                        let servers = self.servers.clone();

                        Task::future(async {
                            get_config_path()
                                .and_then(|path| async move { servers.save(&path).await })
                                .await
                        })
                        .discard()
                    }
                    servercreation::Action::Run(task) => task.map(Message::ServerCreation),
                }
            }
            Message::ServerTerminal(id, message) => {
                let Some(Server {
                    console: Some(console),
                    ..
                }) = self.servers.get_mut(id)
                else {
                    return Task::none();
                };

                let action = ServerTerminal::update(console, message);

                match action {
                    serverboot::Action::None => Task::none(),
                    serverboot::Action::GoBack => {
                        self.screen = Screen::ServerList;

                        Task::none()
                    }
                    serverboot::Action::Run(task) => task.map(Message::ServerTerminal.with(id)),
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

                let Server { info, console, .. } = &self.servers[index];

                ServerTerminal::view(&info.name, console.as_ref().unwrap())
                    .map(move |msg| Message::ServerTerminal(index, msg))
            }
        }
    }
}

async fn get_config_path() -> Result<PathBuf, screen::serverlist::Error> {
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
