use core::str;
use std::{
    fs,
    path::{Path, PathBuf},
};

use iced::{
    advanced::image,
    border, color,
    futures::{SinkExt, Stream, TryFutureExt},
    padding,
    stream::try_channel,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, row,
        rule::{self, FillMode},
        scrollable, svg, text, vertical_rule, vertical_space,
    },
    window, Alignment, Background, Color, ContentFit, Element, Font, Length, Shadow, Subscription,
    Task, Vector,
};
use iced_aw::{drop_down, menu, menu_bar, menu_items, MenuBar};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
};

use iced_aw::menu::{Item, Menu};

use crate::{
    core::{
        metamod::{self, MetamodBranch, MetamodDownloader},
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
    servercreation::{self, FormPage},
};

pub struct State {
    is_server_creation_popup_visible: bool,
    server_creation_screen: servercreation::State,
    pub servers: Vec<Server>,
}

pub struct Server {
    pub info: ServerInfo,
    pub is_running: bool,
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

#[derive(Debug, Clone)]
pub enum Message {
    CreateServer,
    OnClickOutsidePopup,
    ServerCreation(servercreation::Message),
    TestingGrounds(()),
    StartServerTerminal(usize),
    ServerConsoleOpened(usize, window::Id),
    WindowClosed,
    DownloadSourcemod(usize, SourcemodBranch),
    OpenFolder(usize),
    DeleteServer(usize),
    OnServerDeletion(usize),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
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
        #[derive(Deserialize)]
        struct DummyDeserializeStruct {
            servers: Vec<ServerInfo>,
        }

        let mut servers: Vec<Server> = vec![];

        if let Ok(servers_from_config) = fs::read_to_string(PathBuf::from(
            "/home/suza/Coding/Rust/mannager-source/Servers/server_list.toml",
        )) {
            if let Ok(str_config) = toml::from_str::<DummyDeserializeStruct>(&servers_from_config) {
                servers = str_config
                    .servers
                    .into_iter()
                    .map(|server| Server {
                        info: server,
                        is_running: false,
                    })
                    .collect()
            }
        }

        (
            Self {
                is_server_creation_popup_visible: false,
                server_creation_screen: servercreation::State::new(),
                servers,
            },
            Task::none(),
        )
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
                println!("Test1");

                if !self.server_creation_screen.form_info.is_downloading {
                    self.is_server_creation_popup_visible = false;
                    self.server_creation_screen.form_page = FormPage::GameSelection;
                }

                Task::none()
            }
            Message::StartServerTerminal(server_id) => {
                let (_id, open) = window::open(window::Settings {
                    decorations: true,
                    ..window::Settings::default()
                });

                open.map(move |window_id| Message::ServerConsoleOpened(server_id, window_id))
            }
            /*
            Task::run(
                start_server(server_id, &self.servers[server_id]),
                Message::ServerOutput,
            )*/
            Message::ServerCreation(server_creation_message) => match server_creation_message {
                servercreation::Message::FinishServerCreation => {
                    self.is_server_creation_popup_visible = false;

                    self.servers.push(Server {
                        info: ServerInfo {
                            name: self.server_creation_screen.form_info.server_name.clone(),
                            game: self.server_creation_screen.form_info.source_game.clone(),
                            path: self.server_creation_screen.form_info.server_path.clone(),
                            map: self.server_creation_screen.form_info.map_name.clone(),
                            max_players: self.server_creation_screen.form_info.max_players.clone(),
                            password: self.server_creation_screen.form_info.password.clone(),
                        },
                        is_running: false,
                    });

                    #[derive(Deserialize, Serialize)]
                    struct DummyDeserializeStruct {
                        servers: Vec<ServerInfo>,
                    }

                    let fartimus = DummyDeserializeStruct {
                        servers: Vec::from_iter(
                            self.servers.iter().map(|server| server.info.clone()),
                        ),
                    };

                    Task::perform(
                        async move {
                            tokio::fs::write(
                                "/home/suza/Coding/Rust/mannager-source/Servers/server_list.toml",
                                toml::to_string_pretty(&fartimus).unwrap(),
                            )
                            .await
                            .unwrap();
                        },
                        Message::TestingGrounds,
                    )
                }
                _ => self
                    .server_creation_screen
                    .update(server_creation_message)
                    .map(Message::ServerCreation),
            },
            Message::TestingGrounds(_) => Task::none(),
            Message::ServerConsoleOpened(id, window_id) => Task::none(),
            Message::WindowClosed => Task::none(),
            Message::DownloadSourcemod(id, sourcemod_branch) => {
                let path: PathBuf = self.servers[id].info.path.clone();

                let branch = sourcemod_branch.clone();

                println!("Test");

                Task::future(async move {
                    setup_sourcemod(path, branch, SourceEngineVersion::Source1).await
                })
                .discard()
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
                self.servers.remove(id);

                let server_name = self.servers[id].info.name.clone();

                #[derive(Deserialize, Serialize)]
                struct DummyDeserializeStruct {
                    servers: Vec<ServerInfo>,
                }

                let fartimus = DummyDeserializeStruct {
                    servers: Vec::from_iter(self.servers.iter().map(|server| server.info.clone())),
                };

                Task::future(async move {
                    tokio::fs::write(
                        "/home/suza/Coding/Rust/mannager-source/Servers/server_list.toml",
                        toml::to_string_pretty(&fartimus).unwrap(),
                    )
                    .await
                    .unwrap();

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
                            .style(|_theme| text::Style {
                                color: Some(color!(0xffffff))
                            }),
                        horizontal_rule(0),
                        container(scrollable(show_servers(
                            &self.servers.iter().map(|server| &server.info).collect()
                        )))
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

fn show_servers<'a>(servers: &Vec<&'a ServerInfo>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    column(
        servers
            .iter()
            .enumerate()
            .map(|(id, server)| server_entry(id, server)),
    )
    .push(
        button("+")
            .on_press(Message::CreateServer)
            .padding([15, 80]),
    )
    .align_x(Alignment::Center)
    .spacing(10)
    .into()
}

fn server_entry<'a>(id: usize, server: &ServerInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let server_game_image_handle = match server.game {
        SourceAppIDs::TeamFortress2 => svg::Handle::from_path("images/tf2-logo.svg"),
        SourceAppIDs::CounterStrike2 => svg::Handle::from_path("images/cs2-logo.svg"),
        SourceAppIDs::LeftForDead1 => svg::Handle::from_path("images/l4d1-logo.svg"),
        SourceAppIDs::LeftForDead2 => svg::Handle::from_path("images/l4d2-logo.svg"),
    };

    let menu_settings = {
        let sourcemod_sub = Item::with_menu(
            button(text!("Download Sourcemod").color(Color::WHITE))
                .width(Length::Fill)
                .style(|_theme, _status| button::text(_theme, _status)),
            Menu::new(
                [
                    Item::new(
                        button(text!("Stable branch").color(Color::WHITE))
                            .on_press(Message::DownloadSourcemod(id, SourcemodBranch::Stable))
                            .width(Length::Fill)
                            .style(|_theme, _status| button::text(_theme, _status)),
                    ),
                    Item::new(
                        button(text!("Dev branch").color(Color::WHITE))
                            .on_press(Message::DownloadSourcemod(id, SourcemodBranch::Dev))
                            .width(Length::Fill)
                            .style(|_theme, _status| button::text(_theme, _status)),
                    ),
                ]
                .into(),
            )
            .offset(5.0)
            .max_width(200.0),
        );

        MenuBar::new(
            [Item::with_menu(
                button(icon::settings().size(20).color(Color::WHITE))
                    .style(|_theme, _status| button::text(_theme, _status)),
                Menu::new(
                    [
                        sourcemod_sub,
                        Item::new(container(horizontal_rule(1)).padding([0, 10])),
                        Item::new(
                            button(text!("Open folder").color(Color::WHITE))
                                .on_press(Message::OpenFolder(id))
                                .width(Length::Fill)
                                .style(|_theme, _status| button::text(_theme, _status)),
                        ),
                        Item::new(
                            button(text!("Delete server").color(Color::WHITE))
                                .on_press(Message::DeleteServer(id))
                                .width(Length::Fill)
                                .style(|_theme, _status| button::text(_theme, _status)),
                        ),
                    ]
                    .into(),
                )
                .max_width(250.0)
                .offset(5.0),
            )]
            .into(),
        )
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
                text!("{}", server.name.clone())
                    .width(400)
                    .wrapping(text::Wrapping::None)
                    .size(30)
                    .style(|_theme| text::Style {
                        color: Some(color!(0xffffff))
                    }),
                horizontal_space(),
                button(icon::start().size(20).color(Color::WHITE))
                    .on_press(Message::StartServerTerminal(id)),
                menu_settings
            ]
            .spacing(10),
            horizontal_rule(0),
            column![
                text!("Max Players: {}", server.max_players).color(Color::WHITE),
                text!("Map: {}", server.map).color(Color::WHITE),
            ]
        ]
        .padding(padding::left(10))
    ])
    .width(Length::Fill)
    .max_width(720)
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
) -> Result<(), CustomError> {
    MetamodDownloader::download(&path, &MetamodBranch::Stable, &engine)
        .await
        .unwrap();
    SourcemodDownloader::download(&path, &branch, &engine)
        .await
        .unwrap();

    Ok(())
}

#[derive(Debug, Clone)]
pub enum MannagerError {
    MetamodDownloadFailed,
    SourcemodDownloadFailed,
}
