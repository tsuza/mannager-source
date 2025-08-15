use core::str;
use std::{
    f32, fs, io,
    net::Ipv4Addr,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use decoder::Value;
use iced::{
    Alignment, Background, Color, ContentFit, Element, Font, Length, Shadow, Subscription, Task,
    Vector,
    border::{self, radius},
    clipboard, color,
    font::Weight,
    futures::TryFutureExt,
    padding,
    widget::{
        button, center, column, container, horizontal_rule, horizontal_space, hover, progress_bar,
        row,
        rule::{self, FillMode},
        scrollable, svg, text, vertical_rule, vertical_space,
    },
    window,
};

use iced_aw::{
    Menu, MenuBar,
    menu::{DrawPath, Item},
};
use iced_palace::widget::ellipsized_text;

#[cfg(target_os = "windows")]
use iced::advanced::graphics::image::image_rs::ImageFormat;

use snafu::prelude::*;

use crate::ui::{
    CS2_IMAGE, CSS_IMAGE, HL2MP_IMAGE, L4D1_IMAGE, L4D2_IMAGE, NMRIH_IMAGE, TF2_IMAGE,
    style::{
        icon::{location, password, people, plus},
        tf2,
    },
};

use dragking;

use crate::{
    core::{
        Game, SourceEngineVersion,
        metamod::{MetamodBranch, MetamodDownloader},
        sourcemod::{SourcemodBranch, SourcemodDownloader},
    },
    ui::{
        components::{modal::modal, notification::notification},
        style::{self, icon},
    },
};

use super::serverboot::Console;

// use super::{
//     serverboot::{self, DEFAULT_PORT, find_available_port},
//     servercreation::{self, FormPage, Progress, Update, download_server},
// };

const SERVER_LIST_FILE_NAME: &str = "server_list.toml";

#[derive(Debug, Clone)]
pub struct Servers(pub Vec<Server>);

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
}

impl Server {
    pub fn is_running(&self) -> bool {
        self.console.is_some()
    }
}

impl Servers {
    pub async fn load(path: &Path) -> Result<Self, Error> {
        match path.try_exists() {
            Ok(true) => {
                let file_contents = fs::read_to_string(path).unwrap();
                decoder::run(toml::from_str, Servers::decode, &file_contents)
                    .map_err(|_| Error::NoServerListFile)
            }
            _ => Err(Error::NoServerListFile),
        }
    }

    pub fn decode(value: Value) -> Result<Self, decoder::Error> {
        use decoder::decode::{map, sequence};

        let servers: Vec<ServerInfo> = map(value)?
            .optional("servers", sequence(ServerInfo::decode))?
            .unwrap_or_default();

        Ok(Servers(
            servers
                .into_iter()
                .map(|server| Server {
                    info: server,
                    console: None,
                    is_downloading_sourcemod: false,
                })
                .collect(),
        ))
    }

    pub fn encode(&self) -> Value {
        use decoder::encode::sequence;

        let servers = self.iter().map(|server| &server.info);

        sequence(ServerInfo::encode, servers).into()
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
        ])
        .into()
    }
}

pub struct ServerList;

// pub struct Server {
//     pub info: ServerInformation,
//     pub hosted_port: Option<u16>,
//     pub is_downloading_sourcemod: bool,
//     pub is_running: bool,
// }

// pub struct TerminalWindow {
//     pub window_id: Option<window::Id>,
//     pub window_state: serverboot::State,
// }

// impl TerminalWindow {
//     pub fn is_visible(&self) -> bool {
//         self.window_id.is_some()
//     }
// }

pub enum Action {
    None,
    CreateServer,
    UpdateServer(usize),
    EditServer(usize),
    RunServer(usize),
    OpenTerminal(usize),
    StopServer(usize),
    Run(Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateServer,
    UpdateServer(usize),
    EditServer(usize),
    DeleteServer(usize),
    StartServer(usize),
    OpenTerminal(usize),
    StopServer(usize),
    ServerDeleted(usize),
    DownloadSourcemod(usize, SourcemodBranch),
    DownloadSourcemodFinished(usize),
    OpenFolder(usize),
    ServerReorder(dragking::DragEvent),
    CopyServerLinkToClipboard(usize),
    CopyToClipboard(Option<String>),
    // UpdateServerProgress(Update),
    DummyButtonEffectMsg,
}

impl ServerList {
    pub fn update(servers: &mut Servers, message: Message) -> Action {
        match message {
            Message::CreateServer => Action::CreateServer,
            Message::UpdateServer(id) => Action::UpdateServer(id),
            Message::DeleteServer(id) => {
                let Some(server) = servers.get(id) else {
                    return Action::None;
                };

                let path = server.info.path.clone();

                Action::Run(Task::perform(
                    async move {
                        let _ = tokio::fs::remove_dir_all(path).await;
                    },
                    move |_| Message::ServerDeleted(id),
                ))
            }
            Message::ServerDeleted(id) => {
                let server_name = servers.remove(id).info.name;

                Action::Run(
                    Task::future(async move {
                        // let _ = Self::save_server_list_to_file(servers.into_iter()).await;

                        notification(
                            "[ MANNager ] Server Deletion",
                            format!("{server_name} has been successfully deleted."),
                            5,
                        )
                    })
                    .discard(),
                )
            }
            Message::StartServer(id) => Action::RunServer(id),
            Message::StopServer(id) => Action::StopServer(id),
            Message::DownloadSourcemod(id, sourcemod_branch) => {
                let Some(server) = servers.get_mut(id) else {
                    return Action::None;
                };

                if server.is_downloading_sourcemod {
                    return Action::None;
                }

                let path = server.info.path.clone();
                let game = server.info.game.clone();
                let branch = sourcemod_branch.clone();

                server.is_downloading_sourcemod = true;

                Action::Run(Task::perform(
                    async move {
                        let _ =
                            setup_sourcemod(path, game, branch, SourceEngineVersion::Source1).await;
                    },
                    move |_| Message::DownloadSourcemodFinished(id),
                ))
            }
            Message::DownloadSourcemodFinished(id) => {
                let Some(server) = servers.get_mut(id) else {
                    return Action::None;
                };

                server.is_downloading_sourcemod = false;
                let server_name = server.info.name.clone();

                Action::Run(
                    Task::future(async move {
                        notification(
                            "[ MANNager ] Sourcemod Download",
                            format!(
                                "Sourcemod has been successfully downloaded for {server_name}."
                            ),
                            5,
                        )
                    })
                    .discard(),
                )
            }
            Message::OpenFolder(id) => {
                let Some(server) = servers.get(id) else {
                    return Action::None;
                };

                // TODO: Why are we using open::that? And not rfd? Replace
                let _ = open::that(server.info.path.clone());

                Action::None
            }
            Message::ServerReorder(drag_event) => {
                let is_a_server_running = servers
                    .iter()
                    .any(|_server| false /* server.is_running() */);

                if is_a_server_running {
                    return Action::None;
                }

                match drag_event {
                    dragking::DragEvent::Dropped {
                        index,
                        target_index,
                    } => {
                        if target_index != index {
                            servers.swap(index, target_index);

                            return Action::Run(
                                Task::future(async move {
                                    // let _ =
                                    //     Self::save_server_list_to_file(servers.into_iter()).await;
                                })
                                .discard(),
                            );
                        }

                        Action::None
                    }
                    _ => Action::None,
                }
            }
            Message::CopyServerLinkToClipboard(server_id) => {
                let Some(Server {
                    console: Some(console),
                    ..
                }) = servers.get(server_id)
                else {
                    return Action::None;
                };

                let port = console.hosted_port;

                Action::Run(Task::perform(
                    async move {
                        let Ok(ip) = get_public_ip().await else {
                            return None;
                        };

                        Some(format!("{ip}:{port}"))
                    },
                    Message::CopyToClipboard,
                ))
            }
            Message::CopyToClipboard(string) => {
                let Some(string) = string else {
                    return Action::None;
                };

                Action::Run(clipboard::write::<Message>(string).discard())
            }
            Message::EditServer(id) => Action::EditServer(id),
            Message::DummyButtonEffectMsg => Action::None,
            Message::OpenTerminal(id) => Action::OpenTerminal(id),
        }
    }

    pub fn view(servers: &Servers) -> Element<'_, Message> {
        container(column![
            container(
                container(
                    column![
                        text!("Servers")
                            .font(Font::with_name("TF2 Build"))
                            .size(40)
                            .color(Color::WHITE),
                        horizontal_rule(2),
                        container(
                            column![
                                scrollable(show_servers(&servers))
                                    .height(Length::Fill)
                                    .spacing(5),
                                container(
                                    button(
                                        plus()
                                            .size(30)
                                            .width(30)
                                            .align_x(Alignment::Center)
                                            .align_y(Alignment::Center)
                                    )
                                    .on_press(Message::CreateServer)
                                    .padding([15, 20])
                                    .style(
                                        |_theme, _status| {
                                            button::Style {
                                                border: border::rounded(10),
                                                ..style::tf2::Style::button(_theme, _status)
                                            }
                                        }
                                    )
                                )
                                .center_x(Length::Fill)
                            ]
                            .spacing(20)
                        )
                        .padding(padding::top(10))
                    ]
                    .align_x(Alignment::Center)
                )
                .width(1080)
                .height(Length::Fill)
                .padding(padding::all(50).top(20))
                .style(|_theme| style::tf2::Style::primary_container(_theme)
                    .border(border::rounded(3).width(8).color(color!(0x363230))))
            )
            .center_x(Length::Fill)
            .padding(40)
            .height(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::background(color!(0x1c1a19)))
        .into()
    }
}

fn show_servers<'a>(servers: &Servers) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    dragking::column(
        servers
            .iter()
            .enumerate()
            .map(|(id, server)| server_entry(id, server)),
    )
    .on_drag_maybe(
        servers
            .iter()
            .all(|server| !server.is_running() && !server.is_downloading_sourcemod)
            .then_some(Message::ServerReorder),
    )
    .align_x(iced::alignment::Horizontal::Center)
    .spacing(20)
    .style(|_theme| dragking::column::Style {
        ghost_border: border::width(1).rounded(10).color(color!(0x363230)),
        ghost_background: iced::Background::Color(Color {
            a: 0.9,
            ..color!(0x7a716b)
        }),
        ..dragking::column::default(_theme)
    })
    .align_x(Alignment::Center)
    .into()
}

fn server_entry<'a>(id: usize, server: &Server) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let Server {
        info,
        is_downloading_sourcemod,
        ..
    } = &server;

    let server_game_image_handle = match info.game {
        Game::TeamFortress2 => TF2_IMAGE.clone(),
        Game::CounterStrikeSource => CSS_IMAGE.clone(),
        Game::CounterStrike2 => CS2_IMAGE.clone(),
        Game::LeftForDead1 => L4D1_IMAGE.clone(),
        Game::LeftForDead2 => L4D2_IMAGE.clone(),
        Game::HalfLife2DM => HL2MP_IMAGE.clone(),
        Game::NoMoreRoomInHell => NMRIH_IMAGE.clone(),
    };

    let menu_settings = {
        let sourcemod_label = if !is_downloading_sourcemod {
            button(row![
                text!("Download Sourcemod"),
                horizontal_space(),
                icon::right_arrow()
            ])
            .on_press_maybe(if *is_downloading_sourcemod {
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
                    text!("loading"),
                    horizontal_space(),
                    icon::right_arrow()
                ]
                .spacing(10),
            )
            .on_press_maybe((!is_downloading_sourcemod).then_some(Message::DummyButtonEffectMsg))
            .width(Length::Fill)
            .style(|_theme, _status| style::tf2::Style::menu_button(_theme, _status))
        };

        let sourcemod_sub = Item::with_menu(
            sourcemod_label,
            Menu::new(
                [
                    Item::new(
                        button(text!("Stable branch"))
                            .on_press_maybe(if *is_downloading_sourcemod {
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
                            .on_press_maybe(if *is_downloading_sourcemod {
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
                button(icon::menu().size(20).center())
                    .on_press(Message::DummyButtonEffectMsg)
                    .style(|_theme, _status| style::tf2::Style::button(_theme, _status)),
                Menu::new(
                    [
                        Item::new(
                            button("Edit")
                                .on_press(Message::EditServer(id))
                                .width(Length::Fill)
                                .style(|_theme, _status| {
                                    style::tf2::Style::menu_button(_theme, _status)
                                }),
                        ),
                        Item::new(
                            button("Update Server")
                                .on_press(Message::UpdateServer(id))
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
                            button(
                                text!("Delete server").color(Color::from_rgb(0.804, 0.361, 0.361)),
                            )
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
        .draw_path(DrawPath::FakeHovering)
        .padding(0)
        .style(|_theme, _status| style::tf2::Style::menu(_theme, _status))
    };

    let console_button: Option<Element<'a, Message>> = server.is_running().then_some(
        button(icon::terminal().size(20).center())
            .on_press(Message::OpenTerminal(id))
            .style(|_theme, _status| style::tf2::Style::button(_theme, _status))
            .into(),
    );

    let join_link_button: Option<Element<'a, Message>> = server.is_running().then_some(
        button(icon::link().size(20).center())
            .on_press(Message::CopyServerLinkToClipboard(id))
            .style(|_theme, _status| style::tf2::Style::button(_theme, _status))
            .into(),
    );

    let running_button = if !server.is_running() {
        button(icon::start().size(20).center())
            .on_press(Message::StartServer(id))
            .style(|_theme, _status| style::tf2::Style::play_button(_theme, _status))
    } else {
        button(icon::stop().size(20).center())
            .on_press(Message::StopServer(id))
            .style(|_theme, _status| button::danger(_theme, _status))
    };

    let container_color = if server.is_running() {
        color!(0x537321)
    } else {
        color!(0x5b7a8d)
    };

    container(
        container(
            container(
                row![
                    svg(server_game_image_handle)
                        .content_fit(ContentFit::Contain)
                        .width(94)
                        .height(94),
                    vertical_rule(2).style(|_theme| rule::Style {
                        color: Color::from_rgba8(120, 120, 120, 0.4),
                        ..rule::default(_theme)
                    }),
                    column![
                        row![
                            ellipsized_text(format!("{}", &info.name))
                                .wrapping(text::Wrapping::None)
                                .size(25)
                                .font(iced::Font {
                                    weight: Weight::Bold,
                                    ..Font::DEFAULT
                                })
                                .style(|_theme| text::Style {
                                    color: Some(color!(0xffffff))
                                }),
                            horizontal_space(),
                            console_button,
                            join_link_button,
                            running_button,
                            menu_settings
                        ]
                        .spacing(10)
                        .padding(padding::bottom(5))
                        .align_y(Alignment::Center),
                        horizontal_rule(0),
                        row![
                            column![
                                row![
                                    people().color(Color::WHITE),
                                    text!("{}", info.max_players).color(Color::WHITE)
                                ]
                                .align_y(Alignment::Center)
                                .spacing(5),
                                row![
                                    location().color(Color::WHITE),
                                    text!("{}", info.map).color(Color::WHITE)
                                ]
                                .align_y(Alignment::Center)
                                .spacing(5),
                            ]
                            .spacing(5),
                            column![match info.password.as_deref() {
                                Some(password_str) => Element::from(
                                    row![
                                        password().color(Color::WHITE),
                                        hover(
                                            container("").width(100).style(|_theme| {
                                                container::background(Color::BLACK.scale_alpha(0.3))
                                                    .border(border::rounded(2))
                                            }),
                                            text!("{}", password_str).color(Color::WHITE)
                                        )
                                    ]
                                    .align_y(Alignment::Center)
                                    .spacing(5)
                                ),
                                None => Element::from(container("")),
                            }]
                            .spacing(5)
                        ]
                        .spacing(20)
                    ]
                    .padding(padding::left(10))
                ]
                .height(Length::Shrink)
                .spacing(20),
            )
            .width(Length::Fill)
            .padding(10)
            .style(|_theme| container::background(color!(0x2A2725))),
        )
        .padding(10)
        .style(move |_theme| container::background(container_color)),
    )
    .padding(5)
    .style(|_theme| {
        container::background(Color::BLACK).shadow(Shadow {
            color: color!(0, 0, 0, 0.5),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 5.0,
        })
    })
    .into()
}

fn show_update_contianer<'a>(progress: f32) -> Element<'a, Message> {
    container(column![
        text!("Updating the server...")
            .font(Font::with_name("TF2 Build"))
            .size(32)
            .color(Color::WHITE)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        horizontal_rule(0),
        center(progress_bar(0.0..=100.0, progress).girth(20).length(300))
            .width(Length::Fill)
            .height(Length::Fill)
    ])
    .width(720)
    .height(400)
    .padding(10)
    .style(|_theme| style::tf2::Style::primary_container(_theme))
    .into()
}

pub async fn setup_sourcemod(
    path: impl AsRef<Path>,
    game: Game,
    branch: SourcemodBranch,
    engine: SourceEngineVersion,
) -> Result<(), Error> {
    MetamodDownloader::download(&path, &game, &MetamodBranch::Stable, &engine)
        .await
        .context(SourcemodDownloadSnafu)?;
    SourcemodDownloader::download(&path, &game, &branch, &engine)
        .await
        .context(SourcemodDownloadSnafu)?;

    Ok(())
}

pub async fn get_public_ip() -> Result<Ipv4Addr, Error> {
    // The URL of a service that returns the public IP
    let url = "https://api.ipify.org";

    // Send a blocking GET request to fetch the public IP
    let public_ip = reqwest::get(url)
        .map_err(|_| Error::NoPublicIp)
        .await?
        .text()
        .await
        .map_err(|_| Error::NoPublicIp)?;

    Ipv4Addr::from_str(&public_ip).map_err(|_| Error::NoPublicIp)
}

#[derive(Snafu, Debug, Clone)]
pub enum Error {
    #[snafu(display("An error occured while trying to download sourcemod / metamod: {source}"))]
    SourcemodDownloadError { source: crate::core::Error },

    #[snafu(display("Failed to update the server"))]
    ServerUpdateError,

    #[snafu(display("Failed to retrieve the public IP"))]
    NoPublicIp,

    #[snafu(display("Failed to save the server state to the file"))]
    ServerSaveError,

    #[snafu(display("Failed to retrieve the server list file: the file might not exist"))]
    NoServerListFile,

    #[snafu(display("io error: {source}"))]
    Io {
        #[snafu(source(from(io::Error, Arc::new)))]
        source: Arc<io::Error>,
    },
}
