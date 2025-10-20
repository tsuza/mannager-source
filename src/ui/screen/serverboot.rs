use std::{io, net::Ipv4Addr, path::PathBuf, sync::Arc, time::Duration};

use iced::{
    Alignment, Color, Font, Length, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    keyboard, padding,
    stream::try_channel,
    task,
    widget::{button, column, container, row, scrollable, space, text},
};
use portforwarder_rs::port_forwarder::PortMappingProtocol;
use snafu::{ResultExt, Snafu};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};

use iced::widget::text::LineHeight;

use crate::{
    core::portforwarder::{self, PortForwarderIP},
    ui::{
        Element,
        components::{
            notification::notification,
            selectable_text::{self, selectable_text},
            textinput_terminal,
        },
        icons::left_arrow,
        themes::{
            elevation, shadow_from_elevation,
            tf2::{self},
        },
    },
};

pub struct ServerTerminal;

#[derive(Debug, Clone)]
pub struct Console {
    pub output: Vec<TextType>,
    pub input: String,
    pub input_history: Vec<String>,
    pub input_history_index: usize,
    pub handle: task::Handle,
    pub sender: Option<mpsc::Sender<String>>,
    pub hosted_port: u16,
}

impl Console {
    pub fn from_handle(handle: task::Handle, port: u16) -> Self {
        Self {
            output: vec![],
            input: "".to_string(),
            input_history: vec![],
            input_history_index: 0,
            handle: handle.abort_on_drop(),
            sender: None,
            hosted_port: port,
        }
    }

    pub fn start(
        executable_path: PathBuf,
        args: String,
        server_name: String,
        port: u16,
    ) -> impl Stream<Item = Result<ServerCommunicationTwoWay, Error>> {
        try_channel(
            1,
            move |mut output: mpsc::Sender<ServerCommunicationTwoWay>| async move {
                let (sender, mut receiver) = mpsc::channel(100);

                output
                    .send(ServerCommunicationTwoWay::Input(sender))
                    .await
                    .context(ChannelSendSnafu)?;

                #[cfg(target_os = "linux")]
                let mut pty = {
                    let test = pty_process::Pty::new().map_err(|err| Error::SpawnProcessError {
                        msg: err.to_string(),
                    })?;

                    let _ = test.resize(pty_process::Size::new(24, 80));

                    test
                };

                let mut _process = {
                    #[cfg(target_os = "linux")]
                    {
                        pty_process::Command::new(executable_path)
                            .args(args.split_whitespace())
                            .spawn(&pty.pts().map_err(|err| Error::SpawnProcessError {
                                msg: err.to_string(),
                            })?)
                            .map_err(|err| Error::SpawnProcessError {
                                msg: err.to_string(),
                            })?
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
                        "MANNager",
                        "Port forwarding failed.",
                        Duration::from_secs(5),
                    )
                    .await;
                }

                let mut buffer: Vec<u8> = vec![];

                let mut input_bool = false;

                loop {
                    let read_future = process_reader.read_u8();
                    let input_future = receiver.select_next_some();

                    select! {
                        pty_output = read_future => {
                            let byte = pty_output.context(CommunicationSnafu)?;

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

                                    TextType::Input(string)
                                } else {
                                    TextType::Output(string)
                                };

                                let _ = output.send(ServerCommunicationTwoWay::Output(text)).await;

                                buffer.clear();
                            }
                        },

                        input = input_future => {
                            #[cfg(target_os = "linux")]
                            let formatted_string = format!("{}\n", input);

                            #[cfg(target_os = "windows")]
                            let formatted_string = format!("{}", input);

                            let _ = process_writer.write_all(formatted_string.as_bytes()).await;
                            let _ = process_writer.flush().await;

                            input_bool = true;
                        }
                    }
                }
            },
        )
    }
}

pub enum Action {
    None,
    GoBack,
    Run(Task<Message>),
}

#[derive(Clone, Debug)]
pub enum Message {
    ServerTerminalInput(String),
    SubmitServerTerminalInput,
    ShutDownServer,
    OnKeyPress(keyboard::Key, keyboard::Modifiers),
    GoBack,
}

#[derive(Clone, Debug)]
pub enum ServerCommunicationTwoWay {
    Input(mpsc::Sender<String>),
    Output(TextType),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextType {
    Input(String),
    Output(String),
}

pub const DEFAULT_PORT: u16 = 27015;
pub const PORT_OFFSET: u16 = 10;

impl ServerTerminal {
    pub fn update(console: &mut Console, message: Message) -> Action {
        match message {
            Message::ShutDownServer => {
                console.handle.abort();

                Action::None
            }
            Message::ServerTerminalInput(string) => {
                console.input = string;

                Action::None
            }
            Message::SubmitServerTerminalInput => {
                let Some(mut sender) = console.sender.clone() else {
                    return Action::None;
                };

                let input_to_send = console.input.clone();

                if !console.input.is_empty() {
                    console.input_history.push(console.input.clone());
                }

                console.input_history_index = console.input_history.len();

                console.input.clear();

                Action::Run(Task::future(async move { sender.send(input_to_send).await }).discard())
            }
            Message::OnKeyPress(key, _) => {
                let keyboard::Key::Named(key) = key else {
                    return Action::None;
                };

                match key {
                    keyboard::key::Named::ArrowUp => {
                        if console.input_history_index < 1 {
                            return Action::None;
                        }

                        console.input_history_index -= 1;

                        let Some(history_input) =
                            console.input_history.get(console.input_history_index)
                        else {
                            return Action::None;
                        };

                        console.input = history_input.clone();

                        Action::None
                    }
                    keyboard::key::Named::ArrowDown => {
                        if console.input_history_index >= console.input_history.len() {
                            return Action::None;
                        }

                        console.input_history_index += 1;

                        let Some(history_input) =
                            console.input_history.get(console.input_history_index)
                        else {
                            return Action::None;
                        };

                        console.input = history_input.clone();

                        Action::None
                    }
                    _ => Action::None,
                }
            }
            Message::GoBack => Action::GoBack,
        }
    }

    pub fn view<'a>(title: &String, console: &Console) -> Element<'a, Message> {
        let console_output_text = {
            column(console.output.iter().map(|text| {
                match text {
                    TextType::Input(string) => selectable_text(format!("{}", string))
                        .style(|theme| selectable_text::Style {
                            color: Some(Color::from_rgb8(120, 120, 120)),
                            ..tf2::selectable_text::default(theme)
                        })
                        .into(),
                    TextType::Output(string) => selectable_text(format!("{}", string)).into(),
                }
            }))
            .padding(5)
        };

        container(
            column![
                container(
                    row![
                        button(left_arrow().size(20).center()).on_press(Message::GoBack),
                        space::horizontal(),
                        container(
                            text!("{}", title)
                                .font(Font::with_name("TF2 Build"))
                                .size(40)
                                .line_height(1.0)
                                .align_x(Alignment::Center)
                        )
                        .padding(padding::top(4.0).bottom(-3.0)),
                        space::horizontal()
                    ]
                    .width(Length::Fill)
                    .align_y(Alignment::Center)
                    .padding(padding::all(10))
                )
                .align_x(Alignment::Center)
                .style(|theme| tf2::container::outlined(theme)
                    .background(theme.colors().surface.surface_container.lowest)
                    .shadow(shadow_from_elevation(elevation(1), theme.colors().shadow))),
                container(
                    scrollable(console_output_text)
                        .direction(scrollable::Direction::Vertical(
                            scrollable::Scrollbar::new().width(15).scroller_width(12),
                        ))
                        .anchor_bottom()
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(padding::left(10))
                .style(|theme| tf2::container::outlined(theme)
                    .background(theme.colors().surface.surface_container.lowest)
                    .shadow(shadow_from_elevation(elevation(1), theme.colors().shadow))),
                textinput_terminal::TextInput::new("Type your command...", &console.input)
                    .on_input(Message::ServerTerminalInput)
                    .on_submit(Message::SubmitServerTerminalInput)
                    .on_key_press(Message::OnKeyPress)
                    .width(Length::Fill)
                    .line_height(LineHeight::Relative(1.4))
            ]
            .spacing(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .style(|theme| tf2::container::surface(theme))
        .into()
    }
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

#[derive(Snafu, Debug, Clone)]
pub enum Error {
    #[snafu(display("Failed to spawn the process: {msg}"))]
    SpawnProcessError { msg: String },

    #[snafu(display("Error in comunication: {source}"))]
    CommunicationError {
        #[snafu(source(from(io::Error, Arc::new)))]
        source: Arc<io::Error>,
    },

    #[snafu(display("Channel send failed: {source}"))]
    ChannelSendError { source: mpsc::SendError },

    #[snafu(display("There was an error while"))]
    PortForwardingError,

    #[snafu(display("Server path does not exist"))]
    ServerPathError,
}
