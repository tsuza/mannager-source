use core::str;
use std::{fmt::format, fs, path::PathBuf, process::Stdio};

use iced::{
    advanced::image,
    border, color,
    futures::{SinkExt, Stream, StreamExt},
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

use super::serverlist::ServerInfo;

pub struct State {
    running_servers_output: Vec<String>,
    server_running: bool,
    server_terminal_input: String,
    test: Option<pty_process::Pty>,
}

#[derive(Clone, Debug)]
pub enum Message {
    ServerOutput(Result<String, CustomError>),
    ServerTerminalInput(String),
    SendServerTerminalInput,
}

impl State {
    pub fn new(server: &ServerInfo) -> (Self, Task<Message>) {
        (
            Self {
                running_servers_output: vec![],
                server_running: false,
                server_terminal_input: "".into(),
                test: None,
            },
            Task::run(start_server(server), Message::ServerOutput),
        )
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerOutput(string) => {
                self.running_servers_output.push(string.unwrap());

                Task::none()
            }
            Message::ServerTerminalInput(string) => {
                self.server_terminal_input = string;

                Task::none()
            }
            Message::SendServerTerminalInput => {
                self.server_terminal_input = "".into();

                let mut pty = pty_process::Pty::new().unwrap();

                Task::none()
            }
        }
    }

    pub fn subscription(&self, server: &ServerInfo) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<Message> {
        let console_output_text = {
            column(self.running_servers_output.iter().map(|string| {
                text!("{}", string)
                    .style(|_theme| text::Style {
                        color: Some(color!(0xffffff)),
                    })
                    .into()
            }))
            .padding(5)
        };

        container(
            column![
                container(
                    scrollable(console_output_text)
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
                text_input("test", &self.server_terminal_input)
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
        .style(|_theme| container::background(color!(0x3a3430)))
        .into()
    }
}

fn start_server(server: &ServerInfo) -> impl Stream<Item = Result<String, CustomError>> {
    let binary_path = format!("{}/srcds_run", &server.path.to_str().unwrap());

    let args = format!(
        "-console -game tf +map {} +maxplayers {}",
        server.map, server.max_players
    );

    try_channel(1, move |mut output| async move {
        let mut pty = pty_process::Pty::new().unwrap();

        pty.resize(pty_process::Size::new(24, 80)).unwrap();

        let mut process = pty_process::Command::new(binary_path)
            .args(args.split_whitespace())
            .spawn(&pty.pts().unwrap())
            .unwrap();

        let mut out_buf = [0_u8; 4096];

        loop {
            let bytes = pty.read(&mut out_buf).await.unwrap();

            let _ = output
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

#[derive(Debug, Clone)]
pub enum CustomError {
    Null,
}
