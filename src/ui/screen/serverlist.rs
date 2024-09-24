use core::str;
use std::{fs, path::PathBuf};

use iced::{
    advanced::image,
    border, color,
    futures::{SinkExt, Stream},
    padding,
    stream::try_channel,
    widget::{
        button, column, container, horizontal_rule, row,
        rule::{self, FillMode},
        scrollable, svg, text, vertical_space,
    },
    window, Alignment, ContentFit, Element, Length, Shadow, Subscription, Task, Vector,
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
};

use crate::ui::components::modal::modal;

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
    ServerBoot(serverboot::Message),
    StartServerTerminal(usize),
    ServerConsoleOpened(usize, window::Id),
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
        "Mannager - Server List".into()
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
            Message::ServerBoot(message) => todo!(),
            Message::ServerConsoleOpened(id, window_id) => Task::none(),
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
                        text!("Servers").style(|_theme| text::Style {
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
        } else if self.server_running {
            modal(
                base,
                container(server_console(
                    &self.server_terminal_input,
                    &self.running_servers_output,
                ))
                .padding(100),
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

    container(row![
        svg(server_game_image_handle)
            .content_fit(ContentFit::Contain)
            .width(Length::FillPortion(1)),
        column![
            text!("{}", server.name.clone())
                .size(30)
                .style(|_theme| text::Style {
                    color: Some(color!(0xffffff))
                }),
            horizontal_rule(0),
            text!("Max Players: {}", server.max_players).style(|_theme| text::Style {
                color: Some(color!(0xffffff))
            }),
            text!("Map: {}", server.map).style(|_theme| text::Style {
                color: Some(color!(0xffffff))
            }),
            button("Run").on_press(Message::StartServerTerminal(id))
        ]
        .width(Length::FillPortion(3))
    ])
    .width(Length::Fill)
    .max_width(720)
    .height(128)
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
