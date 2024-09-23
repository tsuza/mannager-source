use core::str;
use std::{fmt::format, fs, path::PathBuf, process::Stdio};

use iced::{
    advanced::image,
    border, color,
    futures::{SinkExt, Stream},
    padding,
    stream::try_channel,
    widget::{
        button, center, column, container, horizontal_rule, row,
        rule::{self, FillMode},
        scrollable, svg, text, text_input, vertical_space, TextInput,
    },
    Alignment, ContentFit, Element, Length, Padding, Shadow, Subscription, Task, Vector,
};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
};

use crate::ui::components::modal::modal;

use super::servercreation::{self, FormPage};

pub struct ServerList {
    is_server_creation_popup_visible: bool,
    server_creation_screen: servercreation::State,
    servers: Vec<ServerInfo>,
    running_servers_output: Vec<String>,
    server_running: bool,
    server_terminal_input: String,
    test: pty_process::Pty,
}

#[derive(Deserialize, Serialize, Clone)]
struct ServerInfo {
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
    StartServer(usize),
    ServerOutput(Result<String, CustomError>),
    ServerTerminalInput(String),
    SendServerTerminalInput,
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

impl ServerList {
    pub fn new() -> (Self, Task<Message>) {
        #[derive(Deserialize)]
        struct DummyDeserializeStruct {
            servers: Vec<ServerInfo>,
        }

        let mut servers: Vec<ServerInfo> = vec![];

        if let Ok(servers_from_config) = fs::read_to_string(PathBuf::from(
            "/home/suza/Coding/Rust/mannager-source/Servers/server_list.toml",
        )) {
            if let Ok(str_config) = toml::from_str::<DummyDeserializeStruct>(&servers_from_config) {
                servers = str_config.servers
            }
        }

        (
            Self {
                is_server_creation_popup_visible: false,
                server_creation_screen: servercreation::State::new(),
                servers: servers,
                running_servers_output: vec![],
                server_running: false,
                server_terminal_input: "".into(),
                test: pty_process::Pty::new().unwrap(),
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
            Message::ServerCreation(server_creation_message) => match server_creation_message {
                servercreation::Message::FinishServerCreation => {
                    self.is_server_creation_popup_visible = false;

                    self.servers.push(ServerInfo {
                        name: self.server_creation_screen.form_info.server_name.clone(),
                        game: self.server_creation_screen.form_info.source_game.clone(),
                        path: self.server_creation_screen.form_info.server_path.clone(),
                        map: self.server_creation_screen.form_info.map_name.clone(),
                        max_players: self.server_creation_screen.form_info.max_players.clone(),
                        password: self.server_creation_screen.form_info.password.clone(),
                    });

                    #[derive(Deserialize, Serialize)]
                    struct DummyDeserializeStruct {
                        servers: Vec<ServerInfo>,
                    }

                    let fartimus = DummyDeserializeStruct {
                        servers: Vec::from_iter(self.servers.iter().map(|item| item.clone())),
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
            Message::StartServer(server_id) => {
                self.server_running = true;

                Task::run(
                    start_server(server_id, &self.servers[server_id]),
                    Message::ServerOutput,
                )
            }
            Message::ServerOutput(string) => {
                self.running_servers_output.push(string.unwrap());

                println!("Update message output sent");

                Task::none()
            }
            Message::ServerTerminalInput(string) => {
                self.server_terminal_input = string;

                Task::none()
            }
            Message::SendServerTerminalInput => {
                self.server_terminal_input = "".into();

                let mut pty = pty_process::Pty::new().unwrap();

                self.test = pty;

                Task::none()
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
                        text!("Servers").style(|_theme| text::Style {
                            color: Some(color!(0xffffff))
                        }),
                        horizontal_rule(0),
                        container(scrollable(show_servers(&self.servers)))
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

fn show_servers<'a>(servers: &Vec<ServerInfo>) -> Element<'a, Message>
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
            button("Run").on_press(Message::StartServer(id))
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

fn start_server(id: usize, server: &ServerInfo) -> impl Stream<Item = Result<String, CustomError>> {
    let binary_path = format!("{}/srcds_run", &server.path.to_str().unwrap());

    let args = format!(
        "-console -game tf +map {} +maxplayers {}",
        server.map, server.max_players
    );

    try_channel(id, move |mut output| async move {
        let mut pty = pty_process::Pty::new().unwrap();

        pty.resize(pty_process::Size::new(24, 80)).unwrap();

        let mut process = pty_process::Command::new(binary_path)
            .args(args.split_whitespace())
            .spawn(&pty.pts().unwrap())
            .unwrap();

        let mut out_buf = [0_u8; 4096];

        loop {
            let bytes = pty.read(&mut out_buf).await.unwrap();

            println!("{}", str::from_utf8(&out_buf[..bytes]).unwrap());

            output
                .send(str::from_utf8(&out_buf[..bytes]).unwrap().to_string())
                .await;
        }
        /*
        if let Some(stdout) = process.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            let mut output_clone = output.clone();

            tokio::spawn(async move {
                while let Some(line) = reader.next_line().await.unwrap() {
                    let _ = output_clone.send(line).await;
                }
            });
        }
        if let Some(stderr) = process.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            let mut output_clone = output.clone();

            tokio::spawn(async move {
                while let Some(line) = reader.next_line().await.unwrap() {
                    let _ = output_clone.send(line).await;
                }
            });
        }

        let _ = process.wait().await;

        */

        Ok(())
    })
}

fn server_console<'a>(
    server_terminal_input: &String,
    output: &Vec<String>,
) -> Element<'a, Message> {
    container(
        column![
            container(
                scrollable(
                    column(output.iter().map(|string| {
                        text!("{}", string)
                            .style(|_theme| text::Style {
                                color: Some(color!(0xffffff)),
                            })
                            .into()
                    }))
                    .padding(5)
                )
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new().width(30).scroller_width(25),
                ))
                .anchor_bottom()
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme, _status| scrollable::Style {
                    vertical_rail: scrollable::Rail {
                        background: Some(iced::Background::Color(color!(0x686252))),
                        border: border::rounded(0),
                        scroller: scrollable::Scroller {
                            color: color!(0xada28d),
                            border: border::rounded(0)
                        }
                    },
                    gap: None,
                    ..scrollable::default(_theme, _status)
                }),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::background(color!(0x2a2421)).border(border::rounded(5))),
            text_input("test", server_terminal_input)
                .on_input(Message::ServerTerminalInput)
                .on_submit(Message::SendServerTerminalInput)
                .width(Length::Fill)
                .style(|_theme, _status| text_input::Style {
                    background: iced::Background::Color(color!(0x2a2421)),
                    border: border::width(0),
                    value: color!(0xffffff),
                    ..text_input::default(_theme, _status)
                })
        ]
        .spacing(20),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20)
    .style(|_theme| container::background(color!(0x3a3430)).border(border::rounded(5)))
    .into()
}

#[derive(Debug, Clone)]
pub enum CustomError {
    Null,
}
