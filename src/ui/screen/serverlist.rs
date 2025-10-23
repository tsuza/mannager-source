use core::str;
use std::{
    io,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use iced::{
    Alignment, Color, ContentFit, Font, Function, Length, Task, clipboard,
    font::Weight,
    futures::TryFutureExt,
    padding,
    widget::{
        Button, Text, button, center, column, container, hover, opaque, row, rule, scrollable,
        space, stack, svg, text, text_input,
    },
};
use rfd::FileHandle;

use crate::{
    core::get_arg_game_name,
    ui::{
        components::{metered_progress_bar, tooltip},
        games::SOURCE_GAMES,
        icons::{
            self, left_arrow, link, location, menu, password, people, plus, port, right_arrow,
            start, stop, terminal,
        },
        server::{Server, Servers},
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

use crate::ui::Element;

use dragking;

use crate::{
    core::{
        Game, SourceEngineVersion,
        metamod::{MetamodBranch, MetamodDownloader},
        sourcemod::{SourcemodBranch, SourcemodDownloader},
    },
    ui::components::notification::notification,
};

pub struct ServerList;

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
    ServerReorder(dragking::DragEvent),
    ServerMessage(usize, ServerMessage),
}

#[derive(Debug, Clone)]
pub enum ServerMessage {
    UpdateServer,
    StartEditServer,
    EditServer(EditServer),
    StopEditServer,
    DeleteServer,
    DeleteServerFinished,
    StartServer,
    OpenTerminal,
    StopServer,
    DownloadSourcemod(SourceEngineVersion, SourcemodBranch),
    DownloadSourcemodFinished,
    OpenFolder,
    CopyLink,
    CopyLinkFinished(Option<String>),
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
    ChangeGslt(String),
}

impl ServerList {
    pub fn update(servers: &mut Servers, message: Message) -> Action {
        match message {
            Message::CreateServer => Action::CreateServer,
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
            Message::ServerMessage(id, ServerMessage::UpdateServer) => Action::UpdateServer(id),
            Message::ServerMessage(id, ServerMessage::DeleteServer) => {
                let Some(server) = servers.get(id) else {
                    return Action::None;
                };

                let path = server.info.path.clone();

                servers.remove(id).info.name;

                Action::Run(
                    Task::perform(
                        async move {
                            let _ = tokio::fs::remove_dir_all(path).await;
                        },
                        |_| ServerMessage::DeleteServerFinished,
                    )
                    .map(Message::ServerMessage.with(id)),
                )
            }
            Message::ServerMessage(_, ServerMessage::DeleteServerFinished) => Action::SaveServers,
            Message::ServerMessage(id, ServerMessage::StartServer) => Action::RunServer(id),
            Message::ServerMessage(id, ServerMessage::StopServer) => Action::StopServer(id),
            Message::ServerMessage(
                id,
                ServerMessage::DownloadSourcemod(engine_version, sourcemod_branch),
            ) => {
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

                Action::Run(
                    Task::perform(
                        async move {
                            let _ = setup_sourcemod(path, game, branch, engine_version).await;
                        },
                        |_| ServerMessage::DownloadSourcemodFinished,
                    )
                    .map(Message::ServerMessage.with(id)),
                )
            }
            Message::ServerMessage(id, ServerMessage::DownloadSourcemodFinished) => {
                let Some(server) = servers.get_mut(id) else {
                    return Action::None;
                };

                server.is_downloading_sourcemod = false;
                let server_name = server.info.name.clone();

                Action::Run(
                    Task::future(async move {
                        notification(
                            "MANNager",
                            format!(
                                "Sourcemod has been successfully downloaded for '{server_name}'."
                            ),
                            Duration::from_secs(5),
                        )
                    })
                    .discard(),
                )
            }
            Message::ServerMessage(id, ServerMessage::OpenFolder) => {
                let Some(server) = servers.get(id) else {
                    return Action::None;
                };

                let path = server.info.path.clone();

                Action::Run(
                    Task::future(async {
                        tokio::task::spawn_blocking(|| {
                            let _ = open::that(path);
                        })
                        .await
                    })
                    .discard(),
                )
            }
            Message::ServerMessage(id, ServerMessage::CopyLink) => {
                let Some(Server {
                    console: Some(console),
                    ..
                }) = servers.get(id)
                else {
                    return Action::None;
                };

                let port = console.hosted_port;

                Action::Run(
                    Task::perform(
                        async move {
                            let Ok(ip) = get_public_ip().await else {
                                return None;
                            };

                            Some(format!("{ip}:{port}"))
                        },
                        ServerMessage::CopyLinkFinished,
                    )
                    .map(Message::ServerMessage.with(id)),
                )
            }
            Message::ServerMessage(_, ServerMessage::CopyLinkFinished(string_opt)) => {
                let Some(string) = string_opt else {
                    return Action::None;
                };

                Action::Run(clipboard::write::<Message>(string).discard())
            }
            Message::ServerMessage(id, ServerMessage::StartEditServer) => Action::EditServer(id),
            Message::ServerMessage(id, ServerMessage::EditServer(edit)) => {
                let Some(Server { info, .. }) = servers.get_mut(id) else {
                    return Action::None;
                };

                match edit {
                    EditServer::ChangeName(name) => {
                        info.name = name;

                        Action::None
                    }
                    EditServer::ChangeMap => {
                        let path = PathBuf::from(info.path.display().to_string())
                            .join(get_arg_game_name(&info.game.clone()))
                            .join("maps");

                        Action::Run(
                            Task::perform(
                                rfd::AsyncFileDialog::new()
                                    .set_title("Choose a default map")
                                    .set_directory(path)
                                    .add_filter("Source Map", &["bsp", "vpk"])
                                    .pick_file(),
                                EditServer::ChangeMapFinished,
                            )
                            .map(ServerMessage::EditServer)
                            .map(Message::ServerMessage.with(id)),
                        )
                    }
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
                        info.port = (!port.is_empty())
                            .then(|| port.parse::<u16>().ok())
                            .flatten();

                        Action::None
                    }
                    EditServer::ChangePassword(password) => {
                        info.password = (!password.is_empty()).then_some(password);

                        Action::None
                    }
                    EditServer::ChangeMaxPlayers(max_players) => {
                        info.max_players = max_players;

                        Action::None
                    }
                    EditServer::ChangeGslt(token) => {
                        info.gslt = (!token.is_empty()).then_some(token);

                        Action::None
                    }
                }
            }
            Message::ServerMessage(id, ServerMessage::StopEditServer) => Action::StopEditServer(id),
            Message::ServerMessage(id, ServerMessage::OpenTerminal) => Action::OpenTerminal(id),
            Message::ServerMessage(_, ServerMessage::DummyButtonEffectMsg) => Action::None,
        }
    }

    pub fn view(servers: &Servers) -> Element<'_, Message> {
        let servers = dragking::column(servers.iter().enumerate().map(|(id, server)| {
            if !server.is_editing {
                card(server).map(Message::ServerMessage.with(id))
            } else {
                editable_card(server).map(Message::ServerMessage.with(id))
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
        .align_x(Alignment::Center);

        container(column![
            container(
                container(
                    column![
                        container(
                            text!("Servers")
                                .font(Font::with_name("TF2 Build"))
                                .size(40)
                                .align_x(Alignment::Center)
                                .width(Length::Fill)
                        )
                        .padding(padding::bottom(4)),
                        rule::horizontal(2),
                        container(
                            column![
                                scrollable(servers).height(Length::Fill).spacing(5),
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

fn card<'a>(server: &Server) -> Element<'a, ServerMessage>
where
    Message: Clone + 'a,
{
    let Server {
        info,
        is_downloading_sourcemod,
        ..
    } = &server;

    let menu_settings = {
        fn menu_button<'a>(
            icon: Text<'a, Theme>,
            text: impl text::IntoFragment<'a>,
        ) -> Button<'a, ServerMessage, Theme> {
            button(row![icon, Text::new(text)].spacing(5))
                .width(Length::Fill)
                .style(|theme, status| tf2::button::text(theme, status))
        }

        let sourcemod_label = if !is_downloading_sourcemod {
            button(row![
                text!("Download Sourcemod"),
                space::horizontal(),
                right_arrow()
            ])
            .on_press_maybe(
                (!is_downloading_sourcemod).then_some(ServerMessage::DummyButtonEffectMsg),
            )
            .width(Length::Fill)
            .style(|theme, status| tf2::button::text(theme, status))
        } else {
            button(
                row![
                    text!("Download Sourcemod"),
                    text!("loading"),
                    space::horizontal(),
                    right_arrow()
                ]
                .spacing(10),
            )
            .on_press_maybe(
                (!is_downloading_sourcemod).then_some(ServerMessage::DummyButtonEffectMsg),
            )
            .width(Length::Fill)
            .style(|theme, status| tf2::button::text(theme, status))
        };

        let sourcemod_sub = Item::with_menu(
            sourcemod_label,
            Menu::new(
                [
                    Item::new(
                        menu_button(icons::download(), "Stable branch").on_press_maybe(
                            (!is_downloading_sourcemod).then_some(
                                ServerMessage::DownloadSourcemod(
                                    SourceEngineVersion::Source1,
                                    SourcemodBranch::Stable,
                                ),
                            ),
                        ),
                    ),
                    Item::new(menu_button(icons::download(), "Dev branch").on_press_maybe(
                        (!is_downloading_sourcemod).then_some(ServerMessage::DownloadSourcemod(
                            SourceEngineVersion::Source1,
                            SourcemodBranch::Dev,
                        )),
                    )),
                ]
                .into(),
            )
            .offset(8.0)
            .max_width(200.0),
        );

        MenuBar::new(
            [Item::with_menu(
                button(menu().size(20).center()).on_press(ServerMessage::DummyButtonEffectMsg),
                Menu::new(
                    [
                        Item::new(
                            menu_button(icons::edit(), "Edit")
                                .on_press(ServerMessage::StartEditServer),
                        ),
                        Item::new(
                            menu_button(icons::download(), "Update Server")
                                .on_press(ServerMessage::UpdateServer),
                        ),
                        Item::new(container(rule::horizontal(1)).padding([5, 10])),
                        sourcemod_sub,
                        Item::new(container(rule::horizontal(1)).padding([5, 10])),
                        Item::new(
                            menu_button(icons::folder(), "Open folder")
                                .on_press(ServerMessage::OpenFolder),
                        ),
                        Item::new(
                            menu_button(icons::trash(), "Delete server")
                                .on_press(ServerMessage::DeleteServer),
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
    };

    let console_button = server.is_running().then_some(
        tooltip(
            button(terminal().size(20).center()).on_press(ServerMessage::OpenTerminal),
            "Open the terminal",
            tooltip::Position::Top,
        )
        .delay(Duration::from_millis(500)),
    );

    let join_link_button = server.is_running().then_some(
        tooltip(
            button(link().size(20).center()).on_press(ServerMessage::CopyLink),
            "Copy the server link",
            tooltip::Position::Top,
        )
        .delay(Duration::from_millis(500)),
    );

    let running_button = if !server.is_running() {
        tooltip(
            button(start().size(20).center())
                .on_press(ServerMessage::StartServer)
                .style(|theme, status| tf2::button::success(theme, status)),
            "Start the server",
            tooltip::Position::Top,
        )
        .delay(Duration::from_millis(500))
    } else {
        tooltip(
            button(stop().size(20).center())
                .on_press(ServerMessage::StopServer)
                .style(|theme, status| tf2::button::error(theme, status)),
            "Stop the server",
            tooltip::Position::Top,
        )
        .delay(Duration::from_millis(500))
    };

    let server_image = get_game_image(info.game).unwrap();

    let card = container(
        row![
            svg(server_image)
                .content_fit(ContentFit::Contain)
                .width(94)
                .height(94),
            rule::vertical(2),
            column![
                row![
                    ellipsized_text(format!("{}", &info.name))
                        .wrapping(text::Wrapping::None)
                        .size(25)
                        .font(iced::Font {
                            weight: Weight::Bold,
                            ..Font::DEFAULT
                        }),
                    space::horizontal(),
                    console_button,
                    join_link_button,
                    running_button,
                    menu_settings
                ]
                .spacing(10)
                .padding(padding::bottom(5))
                .align_y(Alignment::Center),
                rule::horizontal(0),
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
                center(
                    metered_progress_bar(0.0..=100.0, percent)
                        .bars(20)
                        .spacing(4)
                        .length(Length::Fill)
                        .girth(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(50)
                .style(|_theme| container::background(Color::BLACK.scale_alpha(0.8)))
            ),
        ]
        .into()
    } else {
        card.into()
    }
}

fn editable_card<'a>(server: &Server) -> Element<'a, ServerMessage>
where
    Message: Clone + 'a,
{
    let Server { info, .. } = &server;

    let game_image = get_game_image(info.game).unwrap();

    let card = container(
        row![
            stack![
                svg(game_image)
                    .content_fit(ContentFit::Contain)
                    .width(94)
                    .height(94),
                button(left_arrow()).on_press(ServerMessage::StopEditServer),
            ],
            rule::vertical(2),
            column![
                row![text_input("Server Name", &info.name).on_input(|string| {
                    ServerMessage::EditServer(EditServer::ChangeName(string))
                })]
                .spacing(10)
                .padding(padding::bottom(5))
                .align_y(Alignment::Center),
                rule::horizontal(0),
                column![
                    row![
                        column![
                            row![
                                people(),
                                number_input(&info.max_players, 0..100, |num| {
                                    ServerMessage::EditServer(EditServer::ChangeMaxPlayers(num))
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
                                .on_input(|port| {
                                    ServerMessage::EditServer(EditServer::ChangePort(port))
                                })
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
                                    .on_press(ServerMessage::EditServer(EditServer::ChangeMap))
                                    .style(|theme, status| tf2::button::outlined(theme, status))
                            ]
                            .align_y(Alignment::Center)
                            .spacing(5),
                            row![
                                password(),
                                text_input(
                                    "Password",
                                    info.password.as_deref().unwrap_or_default()
                                )
                                .on_input(|password| ServerMessage::EditServer(
                                    EditServer::ChangePassword(password)
                                ))
                                .secure(true)
                                .width(200)
                            ]
                            .align_y(Alignment::Center)
                            .spacing(5)
                        ]
                        .width(Length::FillPortion(4))
                        .spacing(5),
                    ]
                    .width(Length::Fill)
                    .spacing(20),
                    row![
                        text("GSLT"),
                        text_input("GSLT", info.gslt.as_deref().unwrap_or_default())
                            .on_input(|token| ServerMessage::EditServer(EditServer::ChangeGslt(
                                token
                            )))
                            .secure(true)
                            .width(320)
                    ]
                    .align_y(Alignment::Center)
                    .spacing(5)
                ]
                .width(Length::Fill)
                .spacing(5)
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

pub fn get_game_image(game: Game) -> Option<svg::Handle> {
    SOURCE_GAMES
        .iter()
        .find(|game_info| game_info.game == game)
        .map(|game_info| game_info.image.clone())
}

pub async fn get_public_ip() -> Result<Ipv4Addr, Error> {
    let url = "https://api.ipify.org";

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
