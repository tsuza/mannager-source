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
    window, Alignment, Color, ContentFit, Element, Font, Length, Shadow, Subscription, Task,
    Vector,
};
use iced_aw::menu::{Item, Menu};
use iced_aw::{menu, style::colors, MenuBar};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};

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

use super::servercreation::{self, FormInfo, FormPage};

const SERVER_LIST_FILE_NAME: &str = "server_list.toml";

pub struct State {
    is_server_creation_popup_visible: bool,
    server_creation_screen: servercreation::State,
    pub servers: Vec<Server>,
    pub images: Images,
}

pub struct Images {
    tf2: svg::Handle,
    l4d1: svg::Handle,
    l4d2: svg::Handle,
    cs2: svg::Handle,
}

pub struct Server {
    pub info: ServerInfo,
    pub is_running: bool,
    pub is_downloading_sourcemod: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub game: SourceAppIDs,
    pub path: PathBuf,
    pub map: String,
    pub max_players: u32,
    pub password: String,
}

impl From<FormInfo> for ServerInfo {
    fn from(form_info: FormInfo) -> Self {
        ServerInfo {
            name: form_info.server_name,
            game: form_info.source_game,
            path: form_info.server_path,
            map: form_info.map_name,
            max_players: form_info.max_players,
            password: form_info.password,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateServer,
    OnClickOutsidePopup,
    ServerCreation(servercreation::Message),
    StartServerTerminal(usize),
    ServerConsoleOpened(usize, window::Id),
    WindowClosed,
    DownloadSourcemod(usize, SourcemodBranch),
    FinishedSourcemodDownload(usize),
    OpenFolder(usize),
    DeleteServer(usize),
    OnServerDeletion(usize),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum SourceAppIDs {
    #[default]
    TeamFortress2,
    CounterStrike2,
    LeftForDead1,
    LeftForDead2,
}

impl From<SourceAppIDs> for u32 {
    fn from(value: SourceAppIDs) -> Self {
        match value {
            SourceAppIDs::TeamFortress2 => 232250,
            SourceAppIDs::CounterStrike2 => 730,
            SourceAppIDs::LeftForDead1 => 222840,
            SourceAppIDs::LeftForDead2 => 222860,
        }
    }
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let mut task = Task::none();

        let servers = Self::get_server_list().unwrap_or_else(|_| {
            task = Task::future(async move {
                Notification::new()
                    .appname("MANNager")
                    .summary("[ MANNager ] Server List")
                    .body("The server list file was not found.")
                    .timeout(5)
                    .show_async()
                    .await
                    .unwrap()
                    .on_close(|_| ())
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
                    tf2: svg::Handle::from_path("images/tf2-logo.svg"),
                    l4d1: svg::Handle::from_path("images/l4d1-logo.svg"),
                    l4d2: svg::Handle::from_path("images/l4d2-logo.svg"),
                    cs2: svg::Handle::from_path("images/cs2-logo.svg"),
                },
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
        println!("test");
        let project_path = directories::ProjectDirs::from("", "MANNager", "mannager-source")
            .ok_or(Error::NoServerListFile)?;

        std::fs::create_dir_all(&project_path.config_dir()).map_err(|_| Error::NoServerListFile)?;

        let config_file = project_path.config_dir().join(SERVER_LIST_FILE_NAME);

        println!("Path: {}", config_file.to_str().unwrap());

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
                is_running: false,
                is_downloading_sourcemod: false,
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
            Message::CreateServer => {
                self.is_server_creation_popup_visible = !self.is_server_creation_popup_visible;

                Task::none()
            }
            Message::OnClickOutsidePopup => {
                if self.server_creation_screen.form_page != FormPage::Downloading
                    && self.server_creation_screen.form_page != FormPage::ServerInfo
                {
                    self.is_server_creation_popup_visible = false;
                    self.server_creation_screen.form_page = FormPage::GameSelection;
                }

                Task::none()
            }
            Message::StartServerTerminal(server_id) => {
                let (_id, open) = window::open(window::Settings::default());

                open.map(move |_| Message::ServerConsoleOpened(server_id, _id))
            }
            Message::ServerCreation(servercreation::Message::FinishServerCreation) => {
                self.is_server_creation_popup_visible = false;

                self.servers.push(Server {
                    info: self.server_creation_screen.form_info.clone().into(),
                    is_running: false,
                    is_downloading_sourcemod: false,
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
            Message::ServerConsoleOpened(_, _) => Task::none(),
            Message::WindowClosed => Task::none(),
            Message::DownloadSourcemod(id, sourcemod_branch) => {
                if self.servers[id].is_downloading_sourcemod {
                    return Task::none();
                }

                let path = self.servers[id].info.path.clone();

                let branch = sourcemod_branch.clone();

                self.servers[id].is_downloading_sourcemod = true;

                Task::perform(
                    async move {
                        let _ = setup_sourcemod(path, branch, SourceEngineVersion::Source1).await;
                    },
                    move |_| Message::FinishedSourcemodDownload(id),
                )
            }
            Message::OpenFolder(id) => {
                let _ = open::that(self.servers[id].info.path.clone());

                Task::none()
            }
            Message::DeleteServer(id) => {
                let path = self.servers[id].info.path.clone();

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
                        .unwrap()
                        .on_close(|_| ())
                })
                .discard()
            }
            Message::FinishedSourcemodDownload(id) => {
                self.servers[id].is_downloading_sourcemod = false;
                let server_name = self.servers[id].info.name.clone();

                Task::future(async move {
                    Notification::new()
                        .appname("MANNager")
                        .summary("[ MANNager ] Sourcemod Download")
                        .body(&format!(
                            "Sourcemod has been successfully downloaded for {server_name}."
                        ))
                        .timeout(5)
                        .show_async()
                        .await
                        .unwrap()
                        .on_close(|_| ());
                })
                .discard()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<Message> {
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
                .style(|_theme| {
                    container::background(color!(0x2A2725))
                        .border(border::width(3).rounded(3).color(color!(0x363230)))
                        .shadow(Shadow {
                            color: color!(0x0),
                            offset: Vector::new(0.0, 3.0),
                            blur_radius: 5.0,
                        })
                })
            )
            .align_x(Alignment::Center)
            .padding(40)
            .width(Length::Fill)
            .height(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::background(color!(0x1c1a19)));

        if self.is_server_creation_popup_visible {
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
    column(
        servers
            .iter()
            .enumerate()
            .map(|(id, server)| server_entry(id, server, images)),
    )
    .push(
        button("+")
            .on_press(Message::CreateServer)
            .padding([15, 80])
            .style(|_theme, _status| style::tf2::Style::button(_theme, _status)),
    )
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
        SourceAppIDs::CounterStrike2 => images.cs2.clone(),
        SourceAppIDs::LeftForDead1 => images.l4d1.clone(),
        SourceAppIDs::LeftForDead2 => images.l4d2.clone(),
    };

    let sourcemod_label = if !server.is_downloading_sourcemod {
        button(row![
            text!("Download Sourcemod"),
            horizontal_space(),
            icon::right_arrow()
        ])
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
        .width(Length::Fill)
        .style(|_theme, _status| style::tf2::Style::menu_button(_theme, _status))
    };

    let menu_settings = {
        let sourcemod_sub = Item::with_menu(
            sourcemod_label,
            Menu::new(
                [
                    Item::new(
                        button(text!("Stable branch"))
                            .on_press(Message::DownloadSourcemod(id, SourcemodBranch::Stable))
                            .width(Length::Fill)
                            .style(|_theme, _status| {
                                style::tf2::Style::menu_button(_theme, _status)
                            }),
                    ),
                    Item::new(
                        button(text!("Dev branch"))
                            .on_press(Message::DownloadSourcemod(id, SourcemodBranch::Dev))
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
                    .style(|_theme, _status| style::tf2::Style::button(_theme, _status)),
                Menu::new(
                    [
                        sourcemod_sub,
                        Item::new(container(horizontal_rule(1)).padding([5, 10])),
                        Item::new(
                            button(text!("Open folder"))
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
        .draw_path(menu::DrawPath::Backdrop)
        .padding(0)
        .style(|_theme, _status| style::tf2::Style::menu(_theme, _status))
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
                    .size(30)
                    .style(|_theme| text::Style {
                        color: Some(color!(0xffffff))
                    }),
                horizontal_space(),
                button(icon::start().size(20).align_y(Alignment::Center))
                    .on_press(Message::StartServerTerminal(id))
                    .style(|_theme, _status| style::tf2::Style::play_button(_theme, _status)),
                menu_settings
            ]
            .spacing(10),
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

#[derive(Debug, Clone)]
pub enum CustomError {
    Null,
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
