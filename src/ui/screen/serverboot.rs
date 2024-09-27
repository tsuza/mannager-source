use iced::{
    border, color,
    futures::{channel::mpsc, SinkExt, Stream, StreamExt},
    padding,
    stream::try_channel,
    task,
    widget::{
        button, center, column, container, horizontal_rule, row,
        rule::{self, FillMode},
        scrollable, svg, text, text_input, vertical_space, TextInput,
    },
    Alignment, Color, ContentFit, Element, Length, Padding, Shadow, Subscription, Task, Vector,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};

use crate::ui::style;

use super::serverlist::ServerInfo;

pub struct State {
    running_servers_output: Vec<String>,
    server_terminal_input: String,
    server_stream_handle: task::Handle,
    sender: Option<mpsc::Sender<String>>,
}

#[derive(Clone, Debug)]
pub enum Message {
    ServerOutput(Result<String, CustomError>),
    ServerCommunication(Result<ServerCommunicationTwoWay, CustomError>),
    ServerTerminalInput(String),
    SendServerTerminalInput,
    ShutDownServer,
}

#[derive(Clone, Debug)]
pub enum ServerCommunicationTwoWay {
    Input(mpsc::Sender<String>),
    Output(String),
}

impl State {
    pub fn new(server: &ServerInfo) -> (Self, Task<Message>) {
        let (task, handle) =
            Task::run(start_server(server), Message::ServerCommunication).abortable();

        (
            Self {
                running_servers_output: vec![],
                server_terminal_input: "".into(),
                server_stream_handle: handle,
                sender: None,
            },
            task,
        )
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerOutput(output) => {
                let Ok(string) = output else {
                    return Task::none();
                };

                self.running_servers_output.push(string);

                Task::none()
            }
            Message::ServerTerminalInput(string) => {
                self.server_terminal_input = string;

                Task::none()
            }
            Message::ShutDownServer => {
                self.server_stream_handle.abort();

                Task::none()
            }
            Message::SendServerTerminalInput => {
                let Some(mut sender) = self.sender.clone() else {
                    return Task::none();
                };

                let input_to_send = self.server_terminal_input.clone();

                self.server_terminal_input = "".to_string();

                Task::future(async move { sender.send(input_to_send).await }).discard()
            }
            Message::ServerCommunication(x) => {
                let Ok(communication) = x else {
                    return Task::none();
                };

                match communication {
                    ServerCommunicationTwoWay::Input(sender) => {
                        self.sender = Some(sender);
                    }
                    ServerCommunicationTwoWay::Output(string) => {
                        self.running_servers_output.push(string);
                    }
                }

                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<Message> {
        let console_output_text = {
            column(
                self.running_servers_output
                    .iter()
                    .map(|string| text!("{}", string).color(Color::WHITE).into()),
            )
            .padding(5)
        };

        container(
            column![
                container(
                    scrollable(console_output_text)
                        .direction(scrollable::Direction::Vertical(
                            scrollable::Scrollbar::new().width(15).scroller_width(12),
                        ))
                        .anchor_bottom()
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|_theme, _status| style::tf2::Style::scrollable(_theme, _status)),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::background(color!(0x2a2421)).border(border::rounded(5))),
                text_input("Type your command...", &self.server_terminal_input)
                    .on_input(Message::ServerTerminalInput)
                    .on_submit(Message::SendServerTerminalInput)
                    .width(Length::Fill)
                    .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
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

fn start_server(
    server: &ServerInfo,
) -> impl Stream<Item = Result<ServerCommunicationTwoWay, CustomError>> {
    let binary_path = format!("{}/srcds_run", &server.path.to_str().unwrap());

    let args = format!(
        "-console -game tf +map cp_dustbowl +maxplayers {} +ip 192.168.178.40 -port 27015",
        server.max_players
    );

    try_channel(1000, move |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);

        output
            .send(ServerCommunicationTwoWay::Input(sender))
            .await
            .unwrap();

        let mut pty = pty_process::Pty::new().unwrap();

        pty.resize(pty_process::Size::new(24, 80)).unwrap();

        let mut _process = pty_process::Command::new(binary_path)
            .args(args.split_whitespace())
            .spawn(&pty.pts().unwrap())
            .unwrap();

        let (mut process_reader, mut process_writer) = pty.split();

        let mut buffer: Vec<u8> = vec![];

        loop {
            let read_future = process_reader.read_u8();
            let input_future = receiver.select_next_some();

            select! {
                pty_output = read_future => {
                    let Ok(byte) = pty_output else {
                        break;
                    };

                    buffer.push(byte);

                    if buffer.len() < 2 {
                        continue;
                    }

                    let Some(last_byte) = buffer.get(buffer.len() - 2) else {
                        continue;
                    };

                    if last_byte == &13u8 && byte == 10u8 {
                        let _ = output.send(ServerCommunicationTwoWay::Output(String::from_utf8(buffer.clone()).unwrap())).await;

                        buffer.clear();
                    }
                },

                input = input_future => {
                    let _ = process_writer.write_all(format!("{}\n\0", input).as_bytes()).await;
                    let _ = process_writer.flush().await;
                }
            }
        }

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum CustomError {
    Null,
}
