use core::str;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use iced::{
    border, color, padding,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, row,
        rule::{self, FillMode},
        scrollable, svg, text, vertical_space,
    },
    window, Alignment, Background, Color, ContentFit, Element, Font, Length, Shadow, Subscription,
    Task, Vector,
};
use iced_aw::{
    menu::{self, Item},
    style::colors,
    Menu, MenuBar,
};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};

use dragking::{self, DropPosition};

use crate::{
    core::{
        metamod::{MetamodBranch, MetamodDownloader},
        sourcemod::{SourcemodBranch, SourcemodDownloader},
        SourceEngineVersion,
    },
    ui::{
        components::modal::modal,
        style::{self, icon},
    },
};

use super::{
    serverboot,
    servercreation::{self, FormInfo, FormPage},
};

const SERVER_LIST_FILE_NAME: &str = "server_list.toml";

pub struct State {
    is_server_creation_popup_visible: bool,
    server_creation_screen: servercreation::State,
    server_edit_screen: Option<(usize, servercreation::State)>,
    pub servers: Vec<Server>,
    pub images: Images,
}

pub struct Images {
    tf2: svg::Handle,
    css: svg::Handle,
    cs2: svg::Handle,
    l4d1: svg::Handle,
    l4d2: svg::Handle,
    nmrih: svg::Handle,
    hl2mp: svg::Handle,
}

pub struct Server {
    pub info: ServerInfo,
    pub is_downloading_sourcemod: bool,
    pub terminal_window: Option<TerminalWindow>,
}

impl Server {
    pub fn is_running(&self) -> bool {
        self.terminal_window.is_some()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub game: SourceAppIDs,
    pub description: String,
    pub path: PathBuf,
    pub map: String,
    pub max_players: u32,
    pub password: String,
    #[serde(default)]
    pub port: u16,
}

pub struct TerminalWindow {
    pub window_id: Option<window::Id>,
    pub window_state: serverboot::State,
}

impl TerminalWindow {
    pub fn is_visible(&self) -> bool {
        self.window_id.is_some()
    }
}

impl From<FormInfo> for ServerInfo {
    fn from(form_info: FormInfo) -> Self {
        ServerInfo {
            name: form_info.server_name,
            game: form_info.source_game,
            description: form_info.server_description,
            path: form_info.server_path,
            map: form_info.map_name,
            max_players: form_info.max_players,
            password: form_info.password,
            port: form_info.port,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ServerTerminalWindowCreated(window::Id, usize),
    WindowClosed,
    TerminalClosed(window::Id),
    CreateServer,
    DeleteServer(usize),
    OnServerDeletion(usize),
    StartServerTerminal(usize),
    CloseServerTerminal(usize),
    DownloadSourcemod(usize, SourcemodBranch),
    FinishedSourcemodDownload(usize),
    OpenFolder(usize),
    ServerReorder(dragking::DragEvent),
    ToggleTerminalWindow(usize),
    OpenEditServerPopup(usize),
    ServerEdit(usize, servercreation::Message),
    ServerTerminal(usize, serverboot::Message),
    ServerCreation(servercreation::Message),
    OnClickOutsidePopup,
    DummyButtonEffectMsg,
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

pub fn get_arg_game_name(game: SourceAppIDs) -> &'static str {
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

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let mut task: Task<Message> = Task::none();

        let servers = Self::get_server_list().unwrap_or_else(|_| {
            task = Task::future(async move {
                Notification::new()
                    .appname("MANNager")
                    .summary("[ MANNager ] Server List")
                    .body("The server list file was not found.")
                    .timeout(5)
                    .show_async()
                    .await
                    .and_then(|notification| Ok(notification.on_close(|_| ())))
            })
            .discard();

            vec![]
        });

        (
            Self {
                is_server_creation_popup_visible: false,
                server_creation_screen: servercreation::State::new(),
                servers,
                images: Images {
                    tf2: svg::Handle::from_memory(include_bytes!("../../../images/tf2-logo.svg")),
                    css: svg::Handle::from_memory(include_bytes!("../../../images/css-logo.svg")),
                    cs2: svg::Handle::from_memory(include_bytes!("../../../images/cs2-logo.svg")),
                    l4d1: svg::Handle::from_memory(include_bytes!("../../../images/l4d1-logo.svg")),
                    l4d2: svg::Handle::from_memory(include_bytes!("../../../images/l4d2-logo.svg")),
                    nmrih: svg::Handle::from_memory(include_bytes!(
                        "../../../images/nmrih-logo.svg"
                    )),
                    hl2mp: svg::Handle::from_memory(include_bytes!(
                        "../../../images/hl2mp-logo.svg"
                    )),
                },
                server_edit_screen: None,
            },
            task,
        )
    }

    fn get_config_file_path() -> Result<PathBuf, Error> {
        if let Ok(config_path) = std::env::current_dir() {
            let config_path = config_path.join(SERVER_LIST_FILE_NAME);

            if config_path.exists() {
                return Ok(config_path);
            }
        }

        let project_path = directories::ProjectDirs::from("", "MANNager", "mannager-source")
            .ok_or(Error::NoServerListFile)?;

        let config_file = project_path.config_dir().join(SERVER_LIST_FILE_NAME);

        match config_file.try_exists()? {
            true => Ok(config_file),
            false => Err(Error::NoServerListFile),
        }
    }

    fn create_config_file_path() -> Result<PathBuf, Error> {
        let project_path = directories::ProjectDirs::from("", "MANNager", "mannager-source")
            .ok_or(Error::NoServerListFile)?;

        std::fs::create_dir_all(&project_path.config_dir()).map_err(|_| Error::NoServerListFile)?;

        let config_file = project_path.config_dir().join(SERVER_LIST_FILE_NAME);

        std::fs::File::create_new(&config_file).map_err(|_| Error::NoServerListFile)?;

        Ok(config_file)
    }

    fn get_server_list() -> Result<Vec<Server>, Error> {
        #[derive(Deserialize)]
        struct ServerList {
            servers: Vec<ServerInfo>,
        }

        let config_path = Self::get_config_file_path()
            .unwrap_or_else(|_| Self::create_config_file_path().unwrap());

        let server_list: ServerList = toml::from_str(&fs::read_to_string(config_path)?)
            .map_err(|_| Error::NoServerListFile)?;

        let servers = server_list
            .servers
            .into_iter()
            .map(|server| Server {
                info: server,
                is_downloading_sourcemod: false,
                terminal_window: None,
            })
            .collect();

        Ok(servers)
    }

    async fn save_server_list_to_file(
        servers: impl Iterator<Item = ServerInfo>,
    ) -> Result<(), Error> {
        #[derive(Deserialize, Serialize)]
        struct ServerList {
            servers: Vec<ServerInfo>,
        }

        let servers = servers.into_iter();

        let fartimus = ServerList {
            servers: Vec::from_iter(servers),
        };

        let config_path = Self::get_config_file_path()
            .unwrap_or_else(|_| Self::create_config_file_path().unwrap());

        tokio::fs::write(
            config_path,
            toml::to_string_pretty(&fartimus).map_err(|_| Error::ServerSaveError)?,
        )
        .await
        .map_err(|_| Error::ServerSaveError)?;

        Ok(())
    }

    pub fn title(&self) -> String {
        "Mannager".into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerTerminalWindowCreated(_window_id, _server_id) => Task::none(),
            Message::WindowClosed => Task::none(),
            Message::TerminalClosed(id) => {
                let server_opt = self.servers.iter_mut().enumerate().find(|(_, server)| {
                    server
                        .terminal_window
                        .as_ref()
                        .map_or(false, |terminal_window| {
                            terminal_window.window_id == Some(id)
                        })
                });

                let Some((server_id, server)) = server_opt else {
                    return Task::none();
                };

                let Some(mut terminal_state) = server.terminal_window.take() else {
                    return Task::none();
                };

                if server.is_running() && !terminal_state.is_visible() {
                    return Task::none();
                }

                terminal_state
                    .window_state
                    .update(serverboot::Message::ShutDownServer)
                    .map(move |x: serverboot::Message| Message::ServerTerminal(server_id, x))
            }
            Message::CreateServer => {
                self.is_server_creation_popup_visible = !self.is_server_creation_popup_visible;

                Task::none()
            }
            Message::DeleteServer(id) => {
                let Some(server) = self.servers.get(id) else {
                    return Task::none();
                };

                let path = server.info.path.clone();

                Task::perform(
                    async move {
                        let _ = tokio::fs::remove_dir_all(path).await;
                    },
                    move |_| Message::OnServerDeletion(id),
                )
            }
            Message::OnServerDeletion(id) => {
                let server_name = self.servers.remove(id).info.name;

                let servers: Vec<ServerInfo> = self
                    .servers
                    .iter()
                    .map(|server| server.info.clone())
                    .collect();

                Task::future(async move {
                    let _ = Self::save_server_list_to_file(servers.into_iter()).await;

                    Notification::new()
                        .appname("MANNager")
                        .summary("[ MANNager ] Server Deletion")
                        .body(&format!("{server_name} has been successfully deleted."))
                        .timeout(5)
                        .show_async()
                        .await
                        .and_then(|notification| Ok(notification.on_close(|_| ())))
                })
                .discard()
            }
            Message::StartServerTerminal(_server_id) => {
                let Some(server) = self.servers.get_mut(_server_id) else {
                    return Task::none();
                };

                if server.is_running() {
                    return Task::none();
                }

                let (_terminal_window_id, _window_task) = window::open(window::Settings::default());
                let (_terminal_state, _terminal_task) = serverboot::State::new(&server.info);

                server.terminal_window = Some(TerminalWindow {
                    window_id: Some(_terminal_window_id),
                    window_state: _terminal_state,
                });

                Task::batch(vec![
                    _window_task.discard(),
                    _terminal_task
                        .map(move |terminal_msg| Message::ServerTerminal(_server_id, terminal_msg)),
                ])
            }
            Message::CloseServerTerminal(_server_id) => {
                let Some(server) = self.servers.get_mut(_server_id) else {
                    return Task::none();
                };

                let Some(window_terminal) = &mut server.terminal_window else {
                    return Task::none();
                };

                let mut tasks = vec![];

                if let Some(window_id) = window_terminal.window_id {
                    window_terminal.window_id = None;

                    tasks.push(window::close(window_id.clone()));
                }

                tasks.push(
                    window_terminal
                        .window_state
                        .update(serverboot::Message::ShutDownServer)
                        .map(move |x| Message::ServerTerminal(_server_id, x)),
                );

                server.terminal_window = None;

                Task::batch(tasks)
            }
            Message::DownloadSourcemod(id, sourcemod_branch) => {
                let Some(server) = self.servers.get_mut(id) else {
                    return Task::none();
                };

                if server.is_downloading_sourcemod {
                    return Task::none();
                }

                let path = server.info.path.clone();

                let branch = sourcemod_branch.clone();

                server.is_downloading_sourcemod = true;

                Task::perform(
                    async move {
                        let _ = setup_sourcemod(path, branch, SourceEngineVersion::Source1).await;
                    },
                    move |_| Message::FinishedSourcemodDownload(id),
                )
            }
            Message::FinishedSourcemodDownload(id) => {
                let Some(server) = self.servers.get_mut(id) else {
                    return Task::none();
                };

                server.is_downloading_sourcemod = false;
                let server_name = server.info.name.clone();

                Task::future(async move {
                    let _ = Notification::new()
                        .appname("MANNager")
                        .summary("[ MANNager ] Sourcemod Download")
                        .body(&format!(
                            "Sourcemod has been successfully downloaded for {server_name}."
                        ))
                        .timeout(5)
                        .show_async()
                        .await
                        .and_then(|notification| Ok(notification.on_close(|_| ())));
                })
                .discard()
            }
            Message::OpenFolder(id) => {
                let Some(server) = self.servers.get(id) else {
                    return Task::none();
                };

                let _ = open::that(server.info.path.clone());

                Task::none()
            }
            Message::ServerReorder(drag_event) => {
                let is_a_server_running = self.servers.iter().any(|server| server.is_running());

                if is_a_server_running {
                    return Task::none();
                }

                match drag_event {
                    dragking::DragEvent::Picked { .. } => Task::none(),
                    dragking::DragEvent::Dropped {
                        index,
                        target_index,
                        drop_position,
                    } => match drop_position {
                        DropPosition::Before => Task::none(),
                        DropPosition::After => Task::none(),
                        DropPosition::Swap => {
                            if target_index != index {
                                self.servers.swap(index, target_index);

                                let servers: Vec<ServerInfo> = self
                                    .servers
                                    .iter()
                                    .map(|server| server.info.clone())
                                    .collect();

                                return Task::future(async move {
                                    let _ =
                                        Self::save_server_list_to_file(servers.into_iter()).await;
                                })
                                .discard();
                            }

                            Task::none()
                        }
                    },
                    dragking::DragEvent::Canceled { .. } => Task::none(),
                }
            }
            Message::ToggleTerminalWindow(server_id) => {
                let Some(server) = self.servers.get_mut(server_id) else {
                    return Task::none();
                };

                let Some(terminal_window) = &mut server.terminal_window else {
                    return Task::none();
                };

                if terminal_window.is_visible() {
                    let Some(window_id) = terminal_window.window_id else {
                        return Task::none();
                    };

                    terminal_window.window_id = None;

                    window::close(window_id)
                } else {
                    let (window_id, window_task) = window::open(window::Settings::default());

                    terminal_window.window_id = Some(window_id);

                    window_task.discard()
                }
            }
            Message::OpenEditServerPopup(server_id) => {
                let Some(server) = self.servers.get(server_id) else {
                    return Task::none();
                };

                self.server_edit_screen = Some((
                    server_id,
                    servercreation::State::from_server_entry(&server.info),
                ));

                Task::none()
            }
            Message::ServerEdit(server_id, servercreation::Message::FinishServerCreation) => {
                let server_edit_screen = self.server_edit_screen.take();

                let Some((_, edit_server_state)) = server_edit_screen else {
                    return Task::none();
                };

                let Some(server) = self.servers.get_mut(server_id) else {
                    return Task::none();
                };

                server.info = edit_server_state.form_info.into();

                let servers: Vec<ServerInfo> = self
                    .servers
                    .iter()
                    .map(|server| server.info.clone())
                    .collect();

                Task::future(Self::save_server_list_to_file(servers.into_iter())).discard()
            }
            Message::ServerEdit(server_id, server_edit_message) => {
                let Some((_, server_edit_state)) = &mut self.server_edit_screen else {
                    return Task::none();
                };

                server_edit_state
                    .update(server_edit_message)
                    .map(move |x| Message::ServerEdit(server_id, x))
            }
            Message::ServerTerminal(id, message) => {
                let Some(server) = self.servers.get_mut(id) else {
                    return Task::none();
                };

                let Some(terminal_window) = &mut server.terminal_window else {
                    return Task::none();
                };

                terminal_window
                    .window_state
                    .update(message)
                    .map(move |msg| Message::ServerTerminal(id, msg))
            }

            Message::ServerCreation(servercreation::Message::FinishServerCreation) => {
                self.is_server_creation_popup_visible = false;

                self.servers.push(Server {
                    info: self.server_creation_screen.form_info.clone().into(),
                    is_downloading_sourcemod: false,
                    terminal_window: None,
                });

                let servers: Vec<ServerInfo> = self
                    .servers
                    .iter()
                    .map(|server| server.info.clone())
                    .collect();

                Task::future(Self::save_server_list_to_file(servers.into_iter())).discard()
            }
            Message::ServerCreation(server_creation_message) => self
                .server_creation_screen
                .update(server_creation_message)
                .map(Message::ServerCreation),
            Message::OnClickOutsidePopup => {
                if self.server_creation_screen.form_page != FormPage::Downloading
                    && self.server_creation_screen.form_page != FormPage::ServerInfo
                {
                    self.is_server_creation_popup_visible = false;
                    self.server_creation_screen.form_page = FormPage::GameSelection;
                }

                self.server_edit_screen = None;

                Task::none()
            }
            Message::DummyButtonEffectMsg => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn view(&self, window_id: window::Id) -> Element<Message> {
        let server_opt = self.servers.iter().enumerate().find(|(_, server)| {
            server
                .terminal_window
                .as_ref()
                .map_or(false, |terminal_window| {
                    terminal_window.window_id == Some(window_id)
                })
        });

        if let Some((server_id, server)) = server_opt {
            if let Some(terminal_window) = &server.terminal_window {
                terminal_window
                    .window_state
                    .view()
                    .map(move |msg| Message::ServerTerminal(server_id, msg))
            } else {
                container("").into()
            }
        } else {
            let base = container(column![
                navbar(),
                container(
                    container(
                        column![
                            text!("Servers")
                                .font(Font::with_name("TF2 Build"))
                                .size(32)
                                .color(Color::WHITE),
                            horizontal_rule(0),
                            container(scrollable(show_servers(&self.servers, &self.images)))
                                .padding(padding::top(20))
                        ]
                        .align_x(Alignment::Center)
                    )
                    .width(900)
                    .padding(padding::all(50).top(10))
                    .style(|_theme| style::tf2::Style::primary_container(_theme))
                )
                .align_x(Alignment::Center)
                .padding(40)
                .width(Length::Fill)
                .height(Length::Fill)
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::background(color!(0x1c1a19)));

            if let Some((server_id, server_edit_state)) = &self.server_edit_screen {
                modal(
                    base,
                    server_edit_state
                        .view()
                        .map(move |x| Message::ServerEdit(*server_id, x)),
                    Message::OnClickOutsidePopup,
                )
            } else if self.is_server_creation_popup_visible {
                modal(
                    base,
                    self.server_creation_screen
                        .view()
                        .map(Message::ServerCreation),
                    Message::OnClickOutsidePopup,
                )
            } else {
                base.into()
            }
        }
    }
}

fn navbar<'a, Message>() -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(column![
        vertical_space(),
        horizontal_rule(0).style(|_theme| rule::Style {
            color: color!(0x363230),
            width: 3,
            fill_mode: FillMode::Full,
            ..rule::default(_theme)
        })
    ])
    .width(Length::Fill)
    .height(64)
    .padding(0)
    .style(|_theme| {
        container::background(color!(0x2A2725)).shadow(Shadow {
            color: color!(0x0),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 5.0,
        })
    })
    .into()
}

fn show_servers<'a>(servers: &Vec<Server>, images: &Images) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    column![
        dragking::column(
            servers
                .iter()
                .enumerate()
                .map(|(id, server)| server_entry(id, server, images)),
        )
        .on_drag_maybe(
            servers
                .iter()
                .all(|server| !server.is_running() && !server.is_downloading_sourcemod)
                .then_some(Message::ServerReorder)
        )
        .align_x(Alignment::Center)
        .spacing(10)
        .style(|_theme| dragking::column::Style {
            ghost_border: border::width(1).rounded(10).color(color!(0x363230)),
            ghost_background: Background::Color(Color {
                a: 0.9,
                ..color!(0x7a716b)
            }),
            ..dragking::column::default(_theme)
        }),
        button("+")
            .on_press(Message::CreateServer)
            .padding([15, 80])
            .style(|_theme, _status| style::tf2::Style::button(_theme, _status)),
    ]
    .align_x(Alignment::Center)
    .spacing(10)
    .into()
}

fn server_entry<'a>(id: usize, server: &Server, images: &Images) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let server_game_image_handle = match server.info.game {
        SourceAppIDs::TeamFortress2 => images.tf2.clone(),
        SourceAppIDs::CounterStrikeSource => images.css.clone(),
        SourceAppIDs::CounterStrike2 => images.cs2.clone(),
        SourceAppIDs::LeftForDead1 => images.l4d1.clone(),
        SourceAppIDs::LeftForDead2 => images.l4d2.clone(),
        SourceAppIDs::HalfLife2DM => images.hl2mp.clone(),
        SourceAppIDs::NoMoreRoomInHell => images.nmrih.clone(),
    };

    let menu_settings = {
        let sourcemod_label = if !server.is_downloading_sourcemod {
            button(row![
                text!("Download Sourcemod"),
                horizontal_space(),
                icon::right_arrow()
            ])
            .on_press_maybe(if server.is_downloading_sourcemod {
                None
            } else {
                Some(Message::DummyButtonEffectMsg)
            })
            .width(Length::Fill)
            .style(|_theme, _status| style::tf2::Style::menu_button(_theme, _status))
        } else {
            button(
                row![
                    text!("Download Sourcemod"),
                    icon::loading(),
                    horizontal_space(),
                    icon::right_arrow()
                ]
                .spacing(10),
            )
            .on_press_maybe(
                (!server.is_downloading_sourcemod).then_some(Message::DummyButtonEffectMsg),
            )
            .width(Length::Fill)
            .style(|_theme, _status| style::tf2::Style::menu_button(_theme, _status))
        };

        let sourcemod_sub = Item::with_menu(
            sourcemod_label,
            Menu::new(
                [
                    Item::new(
                        button(text!("Stable branch"))
                            .on_press_maybe(if server.is_downloading_sourcemod {
                                None
                            } else {
                                Some(Message::DownloadSourcemod(id, SourcemodBranch::Stable))
                            })
                            .width(Length::Fill)
                            .style(|_theme, _status| {
                                style::tf2::Style::menu_button(_theme, _status)
                            }),
                    ),
                    Item::new(
                        button(text!("Dev branch"))
                            .on_press_maybe(if server.is_downloading_sourcemod {
                                None
                            } else {
                                Some(Message::DownloadSourcemod(id, SourcemodBranch::Dev))
                            })
                            .width(Length::Fill)
                            .style(|_theme, _status| {
                                style::tf2::Style::menu_button(_theme, _status)
                            }),
                    ),
                ]
                .into(),
            )
            .offset(8.0)
            .max_width(200.0),
        );

        MenuBar::new(
            [Item::with_menu(
                button(icon::settings().size(20).align_y(Alignment::Center))
                    .on_press(Message::DummyButtonEffectMsg)
                    .style(|_theme, _status| style::tf2::Style::button(_theme, _status)),
                Menu::new(
                    [
                        Item::new(
                            button("Edit")
                                .on_press(Message::OpenEditServerPopup(id))
                                .width(Length::Fill)
                                .style(|_theme, _status| {
                                    style::tf2::Style::menu_button(_theme, _status)
                                }),
                        ),
                        Item::new(container(horizontal_rule(1)).padding([5, 10])),
                        sourcemod_sub,
                        Item::new(container(horizontal_rule(1)).padding([5, 10])),
                        Item::new(
                            button("Open folder")
                                .on_press(Message::OpenFolder(id))
                                .width(Length::Fill)
                                .style(|_theme, _status| {
                                    style::tf2::Style::menu_button(_theme, _status)
                                }),
                        ),
                        Item::new(
                            button(text!("Delete server").color(colors::INDIAN_RED))
                                .on_press(Message::DeleteServer(id))
                                .width(Length::Fill)
                                .style(|_theme, _status| {
                                    style::tf2::Style::menu_button(_theme, _status)
                                }),
                        ),
                    ]
                    .into(),
                )
                .max_width(250.0)
                .offset(5.0),
            )]
            .into(),
        )
        .draw_path(menu::DrawPath::FakeHovering)
        .padding(0)
        .style(|_theme, _status| style::tf2::Style::menu(_theme, _status))
    };

    let toggle_terminal_window: Element<'a, Message> = if server.is_running() {
        button(
            if server
                .terminal_window
                .as_ref()
                .map_or(false, |window| !window.is_visible())
            {
                "Show"
            } else {
                "Hide"
            },
        )
        .on_press_maybe(if server.is_running() {
            Some(Message::ToggleTerminalWindow(id))
        } else {
            None
        })
        .style(|_theme, _status| style::tf2::Style::button(_theme, _status))
        .into()
    } else {
        container("").into()
    };

    let running_button = if !server.is_running() {
        button(icon::start().size(20).align_y(Alignment::Center))
            .on_press(Message::StartServerTerminal(id))
            .style(|_theme, _status| style::tf2::Style::play_button(_theme, _status))
    } else {
        button(icon::stop().size(20).align_y(Alignment::Center))
            .on_press(Message::CloseServerTerminal(id))
            .style(|_theme, _status| button::danger(_theme, _status))
    };

    container(row![
        svg(server_game_image_handle)
            .content_fit(ContentFit::Contain)
            .width(94)
            .height(94),
        column![
            row![
                text!("{}", server.info.name.clone())
                    .width(400)
                    .wrapping(text::Wrapping::None)
                    .size(25)
                    .style(|_theme| text::Style {
                        color: Some(color!(0xffffff))
                    }),
                horizontal_space(),
                toggle_terminal_window,
                running_button,
                menu_settings
            ]
            .spacing(10)
            .padding(padding::bottom(5))
            .align_y(Alignment::Center),
            horizontal_rule(0),
            column![
                text!("Max Players: {}", server.info.max_players).color(Color::WHITE),
                text!("Map: {}", server.info.map).color(Color::WHITE),
            ]
        ]
        .padding(padding::left(10))
    ])
    .width(Length::Fill)
    .padding(10)
    .style(|_theme| {
        container::background(color!(0x2A2725))
            .border(border::width(1).rounded(10).color(color!(0x363230)))
            .shadow(Shadow {
                color: color!(0, 0, 0, 0.5),
                offset: Vector::new(0.0, 3.0),
                blur_radius: 5.0,
            })
    })
    .into()
}

pub async fn setup_sourcemod(
    path: impl AsRef<Path>,
    branch: SourcemodBranch,
    engine: SourceEngineVersion,
) -> Result<(), Error> {
    MetamodDownloader::download(&path, &MetamodBranch::Stable, &engine).await?;
    SourcemodDownloader::download(&path, &branch, &engine).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An error occured while trying to download sourcemod / metamod: {0}")]
    SourcemodDownloadError(#[from] crate::core::Error),

    #[error("Failed to save the server state to the file")]
    ServerSaveError,

    #[error("Failed to retrieve the server list file: the file might not exist")]
    NoServerListFile,

    #[error(transparent)]
    Io(#[from] io::Error),
}
