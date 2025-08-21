use std::{net::Ipv4Addr, sync::LazyLock};

use iced::{Color, Subscription, Task, advanced::svg, clipboard, keyboard};
use screen::{
    Screen,
    serverboot::{
        self, Console, DEFAULT_PORT, PORT_OFFSET, ServerCommunicationTwoWay, ServerTerminal,
        find_available_port,
    },
    servercreation,
    serverlist::{self, Server, ServerList, Servers},
};

#[cfg(target_os = "windows")]
use iced::advanced::graphics::image::image_rs::ImageFormat;

#[cfg(target_os = "linux")]
use crate::{
    core::{Game, get_arg_game_name},
    ui::{components::selectable_text, screen::servercreation::download_server, themes::Theme},
};

#[cfg(target_os = "windows")]
use crate::APP_ICON_BYTES;

pub mod components;
pub mod screen;
pub mod style;
pub mod themes;

const SERVER_LIST_FILE_NAME: &str = "server_list.toml";

static TF2_IMAGE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../images/tf2-logo.svg")));

static CSS_IMAGE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../images/css-logo.svg")));

static CS2_IMAGE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../images/cs2-logo.svg")));

static L4D1_IMAGE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../images/l4d1-logo.svg")));

static L4D2_IMAGE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../images/l4d2-logo.svg")));

static NMRIH_IMAGE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../images/nmrih-logo.svg")));

static HL2MP_IMAGE: LazyLock<svg::Handle> =
    LazyLock::new(|| svg::Handle::from_memory(include_bytes!("../images/hl2mp-logo.svg")));

type Element<'a, Message> = iced::Element<'a, Message, Theme>;

pub struct State {
    screen: Screen,
    servers: Servers,
}

#[derive(Debug, Clone)]
pub enum Message {
    ServersLoaded(Result<Servers, screen::serverlist::Error>),
    ServerList(serverlist::Message),
    ServerCreation(servercreation::Message),
    ServerTerminal(usize, serverboot::Message),
    UpdateServer(usize, Update),
    ServerCommunication(
        usize,
        Result<ServerCommunicationTwoWay, screen::serverboot::Error>,
    ),
    Copy,
    SelectedText(Vec<(f32, String)>),
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
                async move {
                    if let Ok(executable_directory) = std::env::current_dir() {
                        let config_path = executable_directory.join(SERVER_LIST_FILE_NAME);

                        if config_path.try_exists().unwrap_or(false) {
                            return Servers::fetch(&config_path).await;
                        }
                    }

                    let project_path =
                        directories::ProjectDirs::from("", "MANNager", "mannager-source")
                            .ok_or(screen::serverlist::Error::NoServerListFile)?;

                    let config_path = project_path.config_dir().join(SERVER_LIST_FILE_NAME);

                    match config_path.try_exists() {
                        Ok(true) => Servers::fetch(&config_path).await,
                        Ok(false) | Err(_) => Err(serverlist::Error::NoServerListFile),
                    }
                },
                Message::ServersLoaded,
            ),
        )
    }

    pub fn title(&self) -> String {
        match self.screen {
            Screen::Loading | Screen::ServerList => "MANNager".into(),
            Screen::ServerCreation(_) => "MANNager - Creating a server".into(),
            Screen::ServerTerminal(id) => {
                if let Some(Server { info, .. }) = self.servers.get(id) {
                    format!("MANNager - Server Terminal [{}]", info.name)
                } else {
                    "MANNager".into()
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        fn handle_hotkey(key: keyboard::Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
            match key.as_ref() {
                keyboard::Key::Character("c") if _modifiers.command() => Some(Message::Copy),
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

                        Task::none()
                    }
                    servercreation::Action::Run(task) => task.map(Message::ServerCreation),
                }
            }
            Message::ServerList(message) => {
                let action = ServerList::update(&mut self.servers, message);

                match action {
                    serverlist::Action::None => Task::none(),
                    serverlist::Action::SaveServers => {
                        println!("Saving...");
                        if let Ok(executable_directory) = std::env::current_dir() {
                            let config_path = executable_directory.join(SERVER_LIST_FILE_NAME);

                            let servers = self.servers.clone();

                            if config_path.try_exists().unwrap_or(false) {
                                return Task::future(async move {
                                    servers.save(&config_path).await.unwrap()
                                })
                                .discard();
                            }
                        }

                        let project_path =
                            directories::ProjectDirs::from("", "MANNager", "mannager-source")
                                .ok_or(screen::serverlist::Error::NoServerListFile)
                                .unwrap();

                        let config_path = project_path.config_dir().join(SERVER_LIST_FILE_NAME);

                        let servers = self.servers.clone();

                        Task::future(async move { servers.save(&config_path).await.unwrap() })
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
                        .map(move |msg| Message::UpdateServer(id, msg))
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

                        let args = {
                            let mut temp = format!(
                                "-console -game {} +hostname \"{}\" +map {} +maxplayers {} -nohltv -strictportbind +ip 0.0.0.0 -port {} -clientport {}",
                                get_arg_game_name(&info.game),
                                info.name,
                                info.map,
                                info.max_players,
                                port,
                                port + PORT_OFFSET
                            );

                            if info.max_players > 32 && info.game == Game::TeamFortress2 {
                                temp = format!("{temp} -unrestricted_maxplayers");
                            }

                            temp
                        };

                        let (task, handle) = Task::run(
                            Console::start(binary_path, args, info.name.clone(), port),
                            move |msg| Message::ServerCommunication(id, msg),
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
                    serverboot::Action::Run(task) => {
                        task.map(move |msg| Message::ServerTerminal(id, msg))
                    }
                }
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
                    ServerCommunicationTwoWay::Output(string) => {
                        console.output.push(string);
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
