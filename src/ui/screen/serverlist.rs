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
    Alignment, Color, ContentFit, Font, Length, Task, clipboard,
    font::Weight,
    futures::TryFutureExt,
    padding,
    widget::{
        Button, Text, button, center, column, container, horizontal_rule, horizontal_space, hover,
        opaque, progress_bar, row, scrollable, stack, svg, text, text_input, vertical_rule,
    },
};
use rfd::FileHandle;

use crate::{
    core::get_arg_game_name,
    ui::{
        style::icon::{left_arrow, port},
        themes::{Theme, elevation, shadow_from_elevation, tf2},
    },
};

use iced_aw::{
    Menu, MenuBar,
    menu::{DrawPath, Item},
    number_input,
};
use iced_palace::widget::ellipsized_text;

use snafu::prelude::*;

use crate::ui::{
    CS2_IMAGE, CSS_IMAGE, Element, HL2MP_IMAGE, L4D1_IMAGE, L4D2_IMAGE, NMRIH_IMAGE, TF2_IMAGE,
    style::icon::{location, password, people, plus},
};

use dragking;

use crate::{
    core::{
        Game, SourceEngineVersion,
        metamod::{MetamodBranch, MetamodDownloader},
        sourcemod::{SourcemodBranch, SourcemodDownloader},
    },
    ui::{components::notification::notification, style::icon},
};

use super::serverboot::Console;

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
        println!("{}", path.display().to_string());

        let toml = toml::to_string_pretty(&self.encode()).map_err(|_| Error::ServerSaveError)?;

        tokio::fs::write(path, toml)
            .await
            .map_err(|_| Error::ServerSaveError)?;

        println!("Saving done!!");

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
    SaveServers,
    CreateServer,
    UpdateServer(usize),
    EditServer(usize),
    StopEditServer(usize),
    RunServer(usize),
    OpenTerminal(usize),
    StopServer(usize),
    Run(Task<Message>),
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateServer,
    UpdateServer(usize),
    StartEditServer(usize),
    EditServer(usize, EditServer),
    StopEditServer(usize),
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
    DummyButtonEffectMsg,
}

#[derive(Debug, Clone)]
pub enum EditServer {
    ChangeName(String),
    ChangeMap,
    ChangeMapFinished(Option<FileHandle>),
    ChangePassword(String),
    ChangePort(String),
    ChangeMaxPlayers(u32),
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
                servers.remove(id).info.name;

                Action::SaveServers
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
            Message::ServerReorder(drag_event) => match drag_event {
                dragking::DragEvent::Dropped {
                    index,
                    target_index,
                } => {
                    if target_index != index {
                        servers.swap(index, target_index);

                        return Action::SaveServers;
                    }

                    Action::None
                }
                _ => Action::None,
            },
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
            Message::StartEditServer(id) => Action::EditServer(id),
            Message::EditServer(id, edit) => {
                let Some(Server { info, .. }) = servers.get_mut(id) else {
                    return Action::None;
                };

                match edit {
                    EditServer::ChangeName(name) => {
                        info.name = name;

                        Action::None
                    }
                    EditServer::ChangeMap => Action::Run(Task::perform(
                        rfd::AsyncFileDialog::new()
                            .set_title("Choose a default map")
                            .set_directory(format!(
                                "{}/{}/maps",
                                info.path.display().to_string(),
                                get_arg_game_name(&info.game.clone())
                            ))
                            .add_filter("Source Map", &["bsp"])
                            .pick_file(),
                        move |file| Message::EditServer(id, EditServer::ChangeMapFinished(file)),
                    )),
                    EditServer::ChangeMapFinished(file_handle) => {
                        if let Some(file) = file_handle {
                            info.map = file
                                .path()
                                .file_stem()
                                .and_then(|stem| stem.to_str())
                                .and_then(|string| Some(string.to_string()))
                                .unwrap()
                        }

                        Action::None
                    }
                    EditServer::ChangePort(port) => {
                        info.port = if port.is_empty() {
                            None
                        } else {
                            port.parse::<u16>().ok()
                        };

                        Action::None
                    }
                    EditServer::ChangePassword(password) => {
                        info.password = (!password.is_empty()).then(|| password);

                        Action::None
                    }
                    EditServer::ChangeMaxPlayers(max_players) => {
                        info.max_players = max_players;

                        Action::None
                    }
                }
            }
            Message::StopEditServer(id) => Action::StopEditServer(id),
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
                            .align_x(Alignment::Center)
                            .width(Length::Fill),
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
                .style(|_theme| tf2::container::main(_theme))
            )
            .center_x(Length::Fill)
            .padding(40)
            .height(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme| tf2::container::surface(theme))
        .into()
    }
}

fn show_servers<'a>(servers: &Servers) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    dragking::column(servers.iter().enumerate().map(|(id, server)| {
        if !server.is_editing {
            server_entry(id, server)
        } else {
            edit_server_entry(id, server)
        }
    }))
    .on_drag_maybe(
        servers
            .iter()
            .all(|server| {
                !server.is_running()
                    && !server.is_downloading_sourcemod
                    && !server.is_updating()
                    && !server.is_editing
            })
            .then_some(Message::ServerReorder),
    )
    .spacing(20)
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
        fn menu_button<'a>(text: impl text::IntoFragment<'a>) -> Button<'a, Message, Theme> {
            button(Text::new(text))
                .width(Length::Fill)
                .style(|theme, status| tf2::button::text(theme, status))
        }

        let sourcemod_label = if !is_downloading_sourcemod {
            button(row![
                text!("Download Sourcemod"),
                horizontal_space(),
                icon::right_arrow()
            ])
            .on_press_maybe((!is_downloading_sourcemod).then_some(Message::DummyButtonEffectMsg))
            .width(Length::Fill)
            .style(|theme, status| tf2::button::text(theme, status))
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
            .style(|theme, status| tf2::button::text(theme, status))
        };

        let sourcemod_sub = Item::with_menu(
            sourcemod_label,
            Menu::new(
                [
                    Item::new(
                        menu_button("Stable branch")
                            .on_press_maybe((!is_downloading_sourcemod).then_some(
                                Message::DownloadSourcemod(id, SourcemodBranch::Stable),
                            )),
                    ),
                    Item::new(
                        menu_button("Dev branch").on_press_maybe(
                            (!is_downloading_sourcemod)
                                .then_some(Message::DownloadSourcemod(id, SourcemodBranch::Dev)),
                        ),
                    ),
                ]
                .into(),
            )
            .offset(8.0)
            .max_width(200.0),
        );

        MenuBar::new(
            [Item::with_menu(
                button(icon::menu().size(20).center()).on_press(Message::DummyButtonEffectMsg),
                Menu::new(
                    [
                        Item::new(menu_button("Edit").on_press(Message::StartEditServer(id))),
                        Item::new(menu_button("Update Server").on_press(Message::UpdateServer(id))),
                        Item::new(container(horizontal_rule(1)).padding([5, 10])),
                        sourcemod_sub,
                        Item::new(container(horizontal_rule(1)).padding([5, 10])),
                        Item::new(menu_button("Open folder").on_press(Message::OpenFolder(id))),
                        Item::new(menu_button("Delete server").on_press(Message::DeleteServer(id))),
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
    };

    let console_button = server
        .is_running()
        .then_some(button(icon::terminal().size(20).center()).on_press(Message::OpenTerminal(id)));

    let join_link_button = server.is_running().then_some(
        button(icon::link().size(20).center()).on_press(Message::CopyServerLinkToClipboard(id)),
    );

    let running_button = if !server.is_running() {
        button(icon::start().size(20).center()).on_press(Message::StartServer(id))
    } else {
        button(icon::stop().size(20).center()).on_press(Message::StopServer(id))
    };

    let card = container(
        row![
            svg(server_game_image_handle)
                .content_fit(ContentFit::Contain)
                .width(94)
                .height(94),
            vertical_rule(2),
            column![
                row![
                    ellipsized_text(format!("{}", &info.name))
                        .wrapping(text::Wrapping::None)
                        .size(25)
                        .font(iced::Font {
                            weight: Weight::Bold,
                            ..Font::DEFAULT
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
                        row![people(), text!("{}", info.max_players)]
                            .align_y(Alignment::Center)
                            .spacing(5),
                        row![
                            port(),
                            text!(
                                "{}",
                                info.port
                                    .map_or_else(|| "auto".to_string(), |port| port.to_string())
                            )
                        ]
                        .align_y(Alignment::Center)
                        .spacing(5)
                    ]
                    .width(Length::FillPortion(1))
                    .spacing(5),
                    column![
                        row![location(), text!("{}", info.map)]
                            .align_y(Alignment::Center)
                            .spacing(5),
                        info.password.as_deref().map(|password_str| {
                            row![
                                password(),
                                hover(
                                    container("").width(100).style(|_theme| {
                                        container::background(Color::BLACK.scale_alpha(0.2))
                                    }),
                                    text!("{}", password_str)
                                ),
                            ]
                            .align_y(Alignment::Center)
                            .spacing(5)
                        }),
                    ]
                    .width(Length::FillPortion(4))
                    .spacing(5)
                ]
                .width(Length::Fill)
                .spacing(20)
            ]
            .padding(padding::left(10))
        ]
        .spacing(20)
        .height(Length::Shrink),
    )
    .width(Length::Fill)
    .padding(25)
    .style(|theme| {
        tf2::container::outlined(theme)
            .background(theme.colors().surface.surface_container.lowest)
            .shadow(shadow_from_elevation(elevation(1), theme.colors().shadow))
    });

    if let Some(percent) = server.updating_percent {
        stack![
            card,
            opaque(
                center(progress_bar(0.0..=100.0, percent).length(500))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(padding::left(50).right(50))
                    .style(|_theme| container::background(Color::BLACK.scale_alpha(0.8)))
            ),
        ]
        .into()
    } else {
        card.into()
    }
}

fn edit_server_entry<'a>(id: usize, server: &Server) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let Server { info, .. } = &server;

    let server_game_image_handle = match info.game {
        Game::TeamFortress2 => TF2_IMAGE.clone(),
        Game::CounterStrikeSource => CSS_IMAGE.clone(),
        Game::CounterStrike2 => CS2_IMAGE.clone(),
        Game::LeftForDead1 => L4D1_IMAGE.clone(),
        Game::LeftForDead2 => L4D2_IMAGE.clone(),
        Game::HalfLife2DM => HL2MP_IMAGE.clone(),
        Game::NoMoreRoomInHell => NMRIH_IMAGE.clone(),
    };

    let card = container(
        row![
            stack![
                svg(server_game_image_handle)
                    .content_fit(ContentFit::Contain)
                    .width(94)
                    .height(94),
                button(left_arrow()).on_press(Message::StopEditServer(id)),
            ],
            vertical_rule(2),
            column![
                row![
                    text_input("Server Name", &info.name).on_input(move |string| {
                        Message::EditServer(id, EditServer::ChangeName(string))
                    })
                ]
                .spacing(10)
                .padding(padding::bottom(5))
                .align_y(Alignment::Center),
                horizontal_rule(0),
                row![
                    column![
                        row![
                            people(),
                            number_input(&info.max_players, 0..100, move |num| {
                                Message::EditServer(id, EditServer::ChangeMaxPlayers(num))
                            })
                            .width(Length::Fill)
                        ]
                        .align_y(Alignment::Center)
                        .spacing(5),
                        row![
                            port(),
                            text_input(
                                "Port",
                                &info.port.map_or_else(String::new, |port| port.to_string())
                            )
                            .on_input(move |port| Message::EditServer(
                                id,
                                EditServer::ChangePort(port)
                            ))
                        ]
                        .align_y(Alignment::Center)
                        .spacing(5)
                    ]
                    .width(Length::FillPortion(1))
                    .spacing(5),
                    column![
                        row![
                            location(),
                            button(text!("{}", info.map))
                                .on_press(Message::EditServer(id, EditServer::ChangeMap))
                                .style(|theme, status| tf2::button::outlined(theme, status))
                        ]
                        .align_y(Alignment::Center)
                        .spacing(5),
                        row![
                            password(),
                            text_input("Password", info.password.as_deref().unwrap_or_default())
                                .on_input(move |string| Message::EditServer(
                                    id,
                                    EditServer::ChangePassword(string)
                                ))
                                .secure(true)
                                .width(200)
                        ]
                        .align_y(Alignment::Center)
                        .spacing(5)
                    ]
                    .width(Length::FillPortion(4))
                    .spacing(5)
                ]
                .width(Length::Fill)
                .spacing(20)
            ]
            .padding(padding::left(10))
        ]
        .spacing(20)
        .height(Length::Shrink),
    )
    .width(Length::Fill)
    .padding(25)
    .style(|theme| {
        tf2::container::outlined(theme)
            .background(theme.colors().surface.surface_container.lowest)
            .shadow(shadow_from_elevation(elevation(1), theme.colors().shadow))
    });

    card.into()
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
