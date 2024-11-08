use std::{net::Ipv4Addr, path::PathBuf};

use iced::{
    border, color,
    futures::{channel::mpsc, SinkExt, Stream, StreamExt},
    keyboard,
    stream::try_channel,
    task,
    widget::{column, container, scrollable, text},
    Color, Element, Length, Subscription, Task,
};
use iced_aw::style::colors;
use portforwarder_rs::port_forwarder::PortMappingProtocol;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};

use crate::{
    core::{
        get_arg_game_name,
        portforwarder::{self, PortForwarderIP},
        SourceAppIDs,
    },
    ui::{
        components::{notification::notification, textinput_terminal},
        style,
    },
};

use super::serverlist::ServerInfo;

pub struct State {
    running_servers_output: Vec<TerminalText>,
    server_terminal_input: String,
    server_terminal_input_history: Vec<String>,
    server_terminal_input_index: usize,
    server_stream_handle: task::Handle,
    sender: Option<mpsc::Sender<String>>,
}

#[derive(Clone, Debug)]
pub enum Message {
    ServerCommunication(Result<ServerCommunicationTwoWay, Error>),
    ServerTerminalInput(String),
    SubmitServerTerminalInput,
    ShutDownServer,
    OnKeyPress(keyboard::Key, keyboard::Modifiers),
}

#[derive(Clone, Debug)]
pub enum ServerCommunicationTwoWay {
    Input(mpsc::Sender<String>),
    Output(TerminalText),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalText {
    Input(String),
    Output(String),
}

pub const DEFAULT_PORT: u16 = 27015;
pub const PORT_OFFSET: u16 = 10;

#[cfg(target_os = "linux")]
const SRCDS_EXEC_NAME: &str = "srcds_run";

#[cfg(target_os = "windows")]
const SRCDS_EXEC_NAME: &str = "srcds-fix.exe";

impl State {
    pub fn new(server: &ServerInfo, port: u16) -> (Self, Task<Message>) {
        let binary_path = server.path.join(SRCDS_EXEC_NAME);

        let args = {
            let mut temp = format!(
                "-console -game {} +hostname \"{}\" +map {} +maxplayers {} -nohltv -strictportbind +ip 0.0.0.0 -port {} -clientport {}",
                get_arg_game_name(&server.game),
                server.name,
                server.map,
                server.max_players,
                port,
                port + PORT_OFFSET
            );

            if server.max_players > 32 && server.game == SourceAppIDs::TeamFortress2 {
                temp = format!("{temp} -unrestricted_maxplayers");
            }

            temp
        };

        let (task, handle) = Task::run(
            start_server(binary_path, args, server.name.clone(), port),
            Message::ServerCommunication,
        )
        .abortable();

        (
            Self {
                running_servers_output: vec![],
                server_terminal_input: "".into(),
                server_stream_handle: handle.abort_on_drop(),
                sender: None,
                server_terminal_input_history: vec![],
                server_terminal_input_index: 0,
            },
            task,
        )
    }

    pub fn title() -> String {
        "MANNager - Server Terminal".into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShutDownServer => {
                self.server_stream_handle.abort();

                Task::none()
            }
            Message::ServerTerminalInput(string) => {
                self.server_terminal_input = string;

                Task::none()
            }
            Message::SubmitServerTerminalInput => {
                let Some(mut sender) = self.sender.clone() else {
                    return Task::none();
                };

                let input_to_send = self.server_terminal_input.clone();

                if !self.server_terminal_input.is_empty() {
                    self.server_terminal_input_history
                        .push(self.server_terminal_input.clone());
                }

                self.server_terminal_input_index = self.server_terminal_input_history.len();

                self.server_terminal_input.clear();

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
            Message::OnKeyPress(key, _) => {
                let keyboard::Key::Named(key) = key else {
                    return Task::none();
                };

                match key {
                    keyboard::key::Named::ArrowUp => {
                        if self.server_terminal_input_index < 1 {
                            return Task::none();
                        }

                        self.server_terminal_input_index -= 1;

                        let Some(history_input) = self
                            .server_terminal_input_history
                            .get(self.server_terminal_input_index)
                        else {
                            return Task::none();
                        };

                        self.server_terminal_input = history_input.clone();

                        Task::none()
                    }
                    keyboard::key::Named::ArrowDown => {
                        if self.server_terminal_input_index
                            >= self.server_terminal_input_history.len()
                        {
                            return Task::none();
                        }

                        self.server_terminal_input_index += 1;

                        let Some(history_input) = self
                            .server_terminal_input_history
                            .get(self.server_terminal_input_index)
                        else {
                            return Task::none();
                        };

                        self.server_terminal_input = history_input.clone();

                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<Message> {
        let console_output_text = {
            column(self.running_servers_output.iter().map(|text| match text {
                TerminalText::Input(string) => text!("{}", string).color(colors::SILVER).into(),
                TerminalText::Output(string) => text!("{}", string).color(Color::WHITE).into(),
            }))
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
                textinput_terminal::TextInput::new(
                    "Type your command...",
                    &self.server_terminal_input
                )
                .on_input(Message::ServerTerminalInput)
                .on_submit(Message::SubmitServerTerminalInput)
                .on_key_press(Message::OnKeyPress)
                .width(Length::Fill)
                .style(|_theme, _status| style::tf2::Style::server_text_input(_theme, _status))
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
    executable_path: PathBuf,
    args: String,
    server_name: String,
    port: u16,
) -> impl Stream<Item = Result<ServerCommunicationTwoWay, Error>> {
    try_channel(1, move |mut output| async move {
        let (sender, mut receiver) = mpsc::channel(100);

        output
            .send(ServerCommunicationTwoWay::Input(sender))
            .await?;

        #[cfg(target_os = "linux")]
        let mut pty = {
            let test =
                pty_process::Pty::new().map_err(|err| Error::SpawnProcessError(err.to_string()))?;

            let _ = test.resize(pty_process::Size::new(24, 80));

            test
        };

        let mut _process = {
            #[cfg(target_os = "linux")]
            {
                pty_process::Command::new(executable_path)
                    .args(args.split_whitespace())
                    .spawn(
                        &pty.pts()
                            .map_err(|err| Error::SpawnProcessError(err.to_string()))?,
                    )
                    .map_err(|err| Error::SpawnProcessError(err.to_string()))?
            }

            #[cfg(target_os = "windows")]
            {
                use std::process::Stdio;

                tokio::process::Command::new(executable_path)
                    .args(args.split_whitespace())
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .kill_on_drop(true)
                    .creation_flags(0x08000000)
                    .spawn()
                    .map_err(|err| Error::SpawnProcessError(err.to_string()))?
            }
        };

        let (mut process_reader, mut process_writer) = {
            #[cfg(target_os = "linux")]
            {
                pty.split()
            }

            #[cfg(target_os = "windows")]
            {
                (_process.stdout.take().unwrap(), _process.stdin.unwrap())
            }
        };

        let forwarder = portforwarder::PortForwarder::open(
            PortForwarderIP::Any,
            port,
            port,
            PortMappingProtocol::UDP,
            &server_name,
        );

        if let Err(_) = forwarder {
            let _ = notification(
                "[ MANNager ] Server running...",
                "Port forwarding failed.",
                5,
            );
        }

        let mut buffer: Vec<u8> = vec![];

        let mut input_bool = false;

        loop {
            let read_future = process_reader.read_u8();
            let input_future = receiver.select_next_some();

            select! {
                pty_output = read_future => {
                    let byte = pty_output.map_err(|err| Error::CommunicationError(err.to_string()))?;

                    buffer.push(byte);

                    if buffer.len() < 2 {
                        continue;
                    }

                    let Some(_last_byte) = buffer.get(buffer.len() - 2) else {
                        continue;
                    };

                    #[cfg(target_os = "windows")]
                    let line_break = byte == 10u8;

                    #[cfg(target_os = "linux")]
                    let line_break = _last_byte == &13u8 && byte == 10u8;

                    if line_break {
                        let Ok(string) = String::from_utf8(buffer.clone()) else {
                            buffer.clear();

                            continue;
                        };

                        // This is definitely not error proof, but it's the only thing that came to mind.
                        let text = if input_bool {
                            input_bool = false;

                            TerminalText::Input(string)
                        } else {
                            TerminalText::Output(string)
                        };

                        let _ = output.send(ServerCommunicationTwoWay::Output(text)).await;

                        buffer.clear();
                    }
                },

                input = input_future => {
                    #[cfg(target_os = "linux")]
                    let formatted_string = format!("{}\n\0", input);

                    #[cfg(target_os = "windows")]
                    let formatted_string = format!("{}", input);

                    let _ = process_writer.write_all(formatted_string.as_bytes()).await;
                    let _ = process_writer.flush().await;

                    input_bool = true;
                }
            }
        }
    })
}

pub fn find_available_port(ip: Ipv4Addr, starting_port: u16) -> u16 {
    let mut port = starting_port;

    const MAX_ATTEMPTS: u32 = 50;

    let _ip = ip.to_string();

    for _ in 1..MAX_ATTEMPTS {
        match std::net::UdpSocket::bind(format!("{_ip}:{port}"))
            .and_then(|_| std::net::UdpSocket::bind(format!("{_ip}:{}", port + PORT_OFFSET)))
        {
            Ok(_) => {
                break;
            }
            Err(_) => {
                port += 10;

                continue;
            }
        }
    }

    port
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Failed to spawn the process: {0}")]
    SpawnProcessError(String),

    #[error("Error in comunication: {0}")]
    CommunicationError(String),

    #[error("Channel send failed: {0}")]
    ChannelSendError(#[from] mpsc::SendError),

    #[error("There was an error while")]
    PortForwardingError,

    #[error("Server path does not exist")]
    ServerPathError,
}
