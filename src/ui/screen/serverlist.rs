use core::str;
use std::{
    io,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use iced::widget::{
    Space,
    text::{Ellipsis, Wrapping},
};
use iced::{
    Alignment, Color, ContentFit, Font, Function, Length, Task,
    border::{self},
    clipboard,
    font::Weight,
    padding,
    widget::{
        Button, Text, button, center, column, container, hover, opaque, row, rule, scrollable,
        space, stack, svg, text, text_input, tooltip,
    },
};
use rfd::FileHandle;
use sweeten::widget::drag::DragEvent;

use crate::{
    icon,
    ui::{
        components::{
            progress_bar::animated_progress_bar,
            spinner::{Circular, easing},
            toggle_button_group::grouped_buttons,
        },
        games::SOURCE_GAMES,
        server::{HostingMode, Server, Servers},
        themes::{Theme, tf2},
    },
};

use iced_aw::{
    Menu, MenuBar,
    menu::{DrawPath, Item},
    number_input,
};

use snafu::prelude::*;

use crate::ui::Element;

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
    ServerReorder(DragEvent),
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
    HostingModeChange(HostingMode),
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
                DragEvent::Dropped {
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

                servers.remove(id);

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
                // TODO: Look into adding the logic to download the correct verison here
                // insteado of view?
                let Some(server) = servers.get_mut(id) else {
                    return Action::None;
                };

                if server.is_downloading_sourcemod {
                    return Action::None;
                }

                let path = server.info.path.clone();
                let game = server.info.game.clone();
                let branch = sourcemod_branch;

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

                // TODO: This does not account for SDR.
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

                Action::Run(clipboard::write(clipboard::Content::Text(string)).discard())
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
                            .join(&info.game.arg_name())
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
            Message::ServerMessage(id, ServerMessage::HostingModeChange(mode)) => {
                let Some(Server { hosting_mode, .. }) = servers.get_mut(id) else {
                    return Action::None;
                };

                *hosting_mode = mode;

                Action::None
            }
        }
    }

    pub fn view(servers: &Servers) -> Element<'_, Message> {
        let server_amount = servers.len();

        let servers = {
            let server_cards = servers.iter().enumerate().map(|(id, server)| {
                if !server.is_editing {
                    card(server).map(Message::ServerMessage.with(id))
                } else {
                    editable_card(server).map(Message::ServerMessage.with(id))
                }
            });

            let are_servers_idle = servers.iter().all(|server| {
                !server.is_running()
                    && !server.is_downloading_sourcemod
                    && !server.is_updating()
                    && !server.is_editing
            });

            if are_servers_idle {
                sweeten::widget::column(server_cards)
                    .on_drag(Message::ServerReorder)
                    .spacing(10)
                    .align_x(Alignment::Center)
            } else {
                sweeten::widget::column(server_cards)
                    .spacing(10)
                    .align_x(Alignment::Center)
            }
        };

        container(
            container(
                column![
                    container(column![
                        text("Servers")
                            .font(Font::new("TF2 Build"))
                            .size(30)
                            .line_height(1.0)
                            .width(Length::Fill),
                        text!("{server_amount} instances")
                            .size(12)
                            .style(tf2::text::muted)
                    ])
                    .padding(padding::bottom(4)),
                    rule::horizontal(1),
                    column![
                        scrollable(servers).height(Length::Fill).spacing(5),
                        container(
                            button(
                                icon::plus()
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
                    .padding(padding::top(10))
                ]
                .align_x(Alignment::Center),
            )
            .width(1080)
            .height(Length::Fill)
            .padding(padding::all(50).top(20)),
        )
        .center(Length::Fill)
        .style(|theme| tf2::container::main(theme))
        .into()
    }
}

fn card<'a>(server: &'a Server) -> Element<'a, ServerMessage> {
    let Server {
        info,
        is_downloading_sourcemod,
        hosting_mode,
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
                icon::right_arrow()
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
                    Circular::new()
                        .easing(&easing::EMPHASIZED_DECELERATE)
                        .cycle_duration(Duration::from_secs_f32(5.0))
                        .size(20.0),
                    space::horizontal(),
                    icon::right_arrow()
                ]
                .align_y(Alignment::Center)
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
                        menu_button(icon::download(), "Stable branch").on_press_maybe(
                            (!is_downloading_sourcemod).then_some(
                                ServerMessage::DownloadSourcemod(
                                    SourceEngineVersion::Source1,
                                    SourcemodBranch::Stable,
                                ),
                            ),
                        ),
                    ),
                    Item::new(menu_button(icon::download(), "Dev branch").on_press_maybe(
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
                button(icon::menu().size(20).center())
                    .on_press(ServerMessage::DummyButtonEffectMsg),
                Menu::new(
                    [
                        Item::new(
                            menu_button(icon::edit(), "Edit")
                                .on_press(ServerMessage::StartEditServer),
                        ),
                        Item::new(
                            menu_button(icon::download(), "Update Server")
                                .on_press(ServerMessage::UpdateServer),
                        ),
                        Item::new(container(rule::horizontal(1)).padding([5, 10])),
                        sourcemod_sub,
                        Item::new(container(rule::horizontal(1)).padding([5, 10])),
                        Item::new(
                            menu_button(icon::folder(), "Open folder")
                                .on_press(ServerMessage::OpenFolder),
                        ),
                        Item::new(
                            menu_button(icon::trash(), "Delete server")
                                .on_press(ServerMessage::DeleteServer)
                                .style(tf2::button::error),
                        ),
                    ]
                    .into(),
                )
                .max_width(250.0)
                .offset(5.0),
            )]
            .into(),
        )
        .close_on_background_click_global(true)
        .close_on_item_click_global(true)
        .draw_path(DrawPath::Backdrop)
        .padding(0)
    };

    // TODO: Remove the unwrap.
    let server_icon = {
        let icon = get_game_image(info.game).unwrap();

        svg(icon)
            .content_fit(ContentFit::Contain)
            .width(80)
            .height(80)
            .opacity(1.0)
    };

    let header_row = {
        let server_name = column![
            text(info.name.as_str())
                .wrapping(Wrapping::None)
                .ellipsis(Ellipsis::End)
                .size(23)
                .line_height(1.0)
                .width(Length::Fill)
                .font(iced::Font {
                    weight: Weight::Bold,
                    ..Font::DEFAULT
                }),
            text!("{}", info.game)
                .wrapping(Wrapping::None)
                .ellipsis(Ellipsis::End)
                .size(10)
                .line_height(1.0)
                .width(Length::Fill)
                .style(tf2::text::muted)
        ]
        .spacing(5);

        let console_button = server.is_running().then_some(
            button(icon::terminal().size(20).center()).on_press(ServerMessage::OpenTerminal),
        );

        let join_link_button = server.is_running().then_some(
            tooltip(
                button(icon::link().size(20).center()).on_press(ServerMessage::CopyLink),
                "Copy the server link",
                tooltip::Position::Top,
            )
            .delay(Duration::from_millis(500)),
        );

        let running_button = if !server.is_running() {
            button(icon::start().size(20).center())
                .on_press(ServerMessage::StartServer)
                .style(|theme, status| tf2::button::success(theme, status))
        } else {
            button(icon::stop().size(20).center())
                .on_press(ServerMessage::StopServer)
                .style(|theme, status| tf2::button::error(theme, status))
        };

        row![
            server_name,
            console_button,
            join_link_button,
            running_button,
            menu_settings,
        ]
        .spacing(7)
    };

    let info = {
        let can_sdr = SOURCE_GAMES
            .iter()
            .find(|game_info| game_info.game == info.game)
            .map(|game_info| game_info.can_sdr)
            .unwrap();

        let button_group = {
            fn tooltip_icon<'a>(
                icon: Text<'a, Theme>,
                description: &'a str,
            ) -> Element<'a, ServerMessage> {
                tooltip(
                    icon,
                    container(text(description).size(13))
                        .padding(padding::vertical(6).horizontal(10)),
                    tooltip::Position::Top,
                )
                .delay(Duration::from_millis(150))
                .gap(10)
                .style(tf2::container::tooltip)
                .into()
            }

            let mut items = vec![
                (tooltip_icon(icon::local(), "Local"), HostingMode::Local),
                (
                    tooltip_icon(icon::port_forwarding(), "Port Forwarding"),
                    HostingMode::Upnp,
                ),
            ];

            if can_sdr {
                items.push((tooltip_icon(icon::sdr(), "SDR"), HostingMode::Sdr));
            }

            grouped_buttons(
                items,
                *hosting_mode,
                ServerMessage::HostingModeChange,
                tf2::button::default,
            )
        };

        row![
            container(
                row![
                    container(
                        row![
                            icon::users().size(15),
                            text(info.max_players)
                                .ellipsis(Ellipsis::Middle)
                                .wrapping(Wrapping::None)
                                .size(15)
                        ]
                        .align_y(Alignment::Center)
                        .spacing(5)
                    )
                    .padding(padding::horizontal(10).vertical(6))
                    .style(tf2::container::info_container),
                    container(
                        row![
                            icon::port().size(15),
                            text(
                                info.port
                                    .map_or_else(|| "auto".to_string(), |port| port.to_string())
                            )
                            .ellipsis(Ellipsis::Middle)
                            .wrapping(Wrapping::None)
                            .size(15)
                        ]
                        .align_y(Alignment::Center)
                        .spacing(5)
                    )
                    .padding(padding::horizontal(10).vertical(6))
                    .style(tf2::container::info_container),
                    container(
                        row![
                            icon::map().size(15),
                            text(info.map.as_str())
                                .ellipsis(Ellipsis::Middle)
                                .wrapping(Wrapping::None)
                                .size(15)
                        ]
                        .align_y(Alignment::Center)
                        .spacing(5)
                    )
                    .padding(padding::horizontal(10).vertical(6))
                    .style(tf2::container::info_container),
                    info.password.as_deref().map(|password_str| {
                        container(
                            row![
                                icon::password().size(15),
                                hover(
                                    container("").width(100).style(|_theme| {
                                        container::background(Color::BLACK.scale_alpha(0.2))
                                    }),
                                    iced_selection::text(password_str)
                                        .ellipsis(Ellipsis::Middle)
                                        .wrapping(Wrapping::None)
                                        .size(15)
                                ),
                            ]
                            .align_y(Alignment::Center)
                            .spacing(5),
                        )
                        .padding(padding::horizontal(10).vertical(6))
                        .style(tf2::container::info_container)
                    }),
                ]
                .spacing(20)
            )
            .width(Length::Fill),
            column![
                text("NETWORK").size(10).style(tf2::text::muted),
                button_group
            ]
        ]
        .align_y(Alignment::End)
        .spacing(20)
    };

    // TODO: Change the colors into the theme's, thus using the ones for the play/stop button
    let status_bar = {
        let color = |theme: &Theme| {
            if server.is_running() {
                theme.colors().success.color
            } else {
                theme.colors().error.color
            }
        };

        container(
            container(Space::new())
                .height(Length::Fill)
                .width(Length::Fill)
                .style(move |theme| {
                    container::background(color(theme)).border(
                        border::rounded(border::left(8))
                            .width(1)
                            .color(color(theme)),
                    )
                }),
        )
        .width(10)
    };

    let card = container(
        row![
            status_bar,
            row![server_icon, column![header_row, info].spacing(10)]
                .align_y(Alignment::Center)
                .spacing(20)
                .padding(padding::vertical(12).horizontal(14)),
        ]
        .align_y(Alignment::Center),
    )
    .width(Length::Fill) // TODO: make it 550 and put two per row
    .style(tf2::container::card);

    if let Some(percent) = server.updating_percent {
        stack![
            card,
            opaque(
                center(
                    animated_progress_bar(0.0..=100.0, percent)
                        .length(Length::Fill)
                        .girth(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(padding::all(30).horizontal(50))
                .style(|_theme| container::background(Color::BLACK.scale_alpha(0.8)))
            ),
        ]
        .into()
    } else {
        card.into()
    }
}

fn editable_card<'a>(server: &'a Server) -> Element<'a, ServerMessage> {
    let Server { info, .. } = &server;

    // TODO: Remove the unwrap.
    let server_icon = {
        let icon = get_game_image(info.game).unwrap();

        svg(icon)
            .content_fit(ContentFit::Contain)
            .width(80)
            .height(80)
            .opacity(1.0)
    };

    let header_row = {
        let server_name = text_input("Name", &info.name)
            .on_input(|string| ServerMessage::EditServer(EditServer::ChangeName(string)))
            .size(25)
            .line_height(1.0)
            .width(Length::Fill)
            .font(iced::Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            });

        server_name
    };

    let info = {
        row![
            container(
                column![
                    row![
                        container(
                            row![
                                icon::users().size(15),
                                number_input(&info.max_players, 0..100, |num| {
                                    ServerMessage::EditServer(EditServer::ChangeMaxPlayers(num))
                                })
                                .set_size(15)
                            ]
                            .align_y(Alignment::Center)
                            .spacing(5)
                        )
                        .padding(padding::horizontal(10).vertical(6))
                        .style(tf2::container::info_container),
                        container(
                            row![
                                icon::port().size(15),
                                text_input(
                                    "Port",
                                    &info.port.map_or_else(String::new, |port| port.to_string())
                                )
                                .on_input(|port| {
                                    ServerMessage::EditServer(EditServer::ChangePort(port))
                                })
                                .size(15)
                            ]
                            .align_y(Alignment::Center)
                            .spacing(5)
                        )
                        .padding(padding::horizontal(10).vertical(6))
                        .style(tf2::container::info_container),
                        container(
                            row![
                                icon::map().size(15),
                                button(text(info.map.as_str()).size(15))
                                    .on_press(ServerMessage::EditServer(EditServer::ChangeMap))
                            ]
                            .align_y(Alignment::Center)
                            .spacing(5)
                        )
                        .padding(padding::horizontal(10).vertical(6))
                        .style(tf2::container::info_container),
                        container(
                            row![
                                icon::password().size(15),
                                text_input(
                                    "Password",
                                    info.password.as_deref().unwrap_or_default()
                                )
                                .on_input(|password| ServerMessage::EditServer(
                                    EditServer::ChangePassword(password)
                                ))
                                .secure(true)
                                .size(15)
                            ]
                            .align_y(Alignment::Center)
                            .spacing(5),
                        )
                        .padding(padding::horizontal(10).vertical(6))
                        .style(tf2::container::info_container)
                    ]
                    .spacing(20),
                    container(
                        row![
                            text("GSLT").size(15),
                            text_input("GSLT", info.gslt.as_deref().unwrap_or_default())
                                .on_input(|token| ServerMessage::EditServer(
                                    EditServer::ChangeGslt(token)
                                ))
                                .secure(true)
                                .size(15)
                        ]
                        .spacing(5)
                        .align_y(Alignment::Center)
                    )
                    .padding(padding::horizontal(10).vertical(6))
                    .style(tf2::container::info_container)
                ]
                .spacing(12)
            )
            .width(Length::Fill),
        ]
        .align_y(Alignment::End)
        .spacing(20)
    };

    // TODO: Change the colors into the theme's, thus using the ones for the play/stop button
    let status_bar = {
        let color = |theme: &Theme| {
            if server.is_running() {
                theme.colors().success.color
            } else {
                theme.colors().error.color
            }
        };

        container(
            container(Space::new())
                .height(Length::Fill)
                .width(Length::Fill)
                .style(move |theme| {
                    container::background(color(theme)).border(
                        border::rounded(border::left(8))
                            .width(1)
                            .color(color(theme)),
                    )
                }),
        )
        .width(10)
    };

    stack![
        container(
            row![
                status_bar,
                row![server_icon, column![header_row, info].spacing(10)]
                    .align_y(Alignment::Center)
                    .spacing(20)
                    .padding(padding::vertical(12).horizontal(14)),
            ]
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .style(tf2::container::card),
        container(
            button(
                icon::left_arrow()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .size(20)
                    .center()
            )
            .on_press(ServerMessage::StopEditServer)
        )
        .width(Length::Shrink)
        .height(Length::Shrink)
        .padding(padding::vertical(12).horizontal(14))
    ]
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

pub fn get_game_image(game: Game) -> Option<svg::Handle> {
    SOURCE_GAMES
        .iter()
        .find(|game_info| game_info.game == game)
        .map(|game_info| game_info.image.clone())
}

pub async fn get_public_ip() -> Result<Ipv4Addr, Error> {
    let url = "https://api.ipify.org";

    let public_ip = reqwest::get(url)
        .await
        .map_err(|_| Error::NoPublicIp)?
        .text()
        .await
        .map_err(|_| Error::NoPublicIp)?;

    public_ip
        .trim()
        .parse::<Ipv4Addr>()
        .map_err(|_| Error::NoPublicIp)
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
