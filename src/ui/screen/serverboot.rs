use std::{
    io,
    net::{Ipv4Addr, UdpSocket},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

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
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    select,
};

use iced::widget::text::LineHeight;

use crate::{
    core::portforwarder::{self, PortForwarderIP},
    icon,
    ui::{
        Element,
        components::{notification::notification, textinput_terminal},
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
                    let pty = pty_process::Pty::new().map_err(|err| Error::SpawnProcessError {
                        msg: err.to_string(),
                    })?;

                    let _ = pty.resize(pty_process::Size::new(24, 80));

                    pty
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

                        const CREATE_NO_WINDOW: u32 = 0x08000000;

                        tokio::process::Command::new(executable_path)
                            .args(args.split_whitespace())
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .kill_on_drop(true)
                            .creation_flags(CREATE_NO_WINDOW)
                            .spawn()
                            .map_err(|err| Error::SpawnProcessError {
                                msg: err.to_string(),
                            })?
                    }
                };

                let (process_reader, mut process_writer) = {
                    #[cfg(target_os = "linux")]
                    {
                        pty.split()
                    }

                    #[cfg(target_os = "windows")]
                    {
                        (_process.stdout.take().unwrap(), _process.stdin.unwrap())
                    }
                };

                let mut reader = BufReader::new(process_reader);
                let mut line = String::new();

                let mut input_bool = false;

                loop {
                    line.clear();

                    let read_future = reader.read_line(&mut line);
                    let input_future = receiver.select_next_some();

                    select! {
                        pty_output = read_future => {
                            let bytes = pty_output.context(CommunicationSnafu)?;

                            if bytes == 0 {
                                continue;
                            }

                            // This is definitely not error proof, but it's the only thing that came to mind.
                            let text = if input_bool {
                                input_bool = false;

                                TextType::Input(line.trim_end().to_owned())
                            } else {
                                TextType::Output(line.trim_end().to_owned())
                            };

                            let _ = output.send(ServerCommunicationTwoWay::Output(text)).await;
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

    pub async fn port_forward(server_name: String, port: u16) {
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
        let header = container(
            row![
                button(icon::left_arrow().size(20).center()).on_press(Message::GoBack),
                space::horizontal(),
                container(
                    text!("{}", title)
                        .font(Font::new("TF2 Build"))
                        .size(40)
                        .line_height(1.0)
                )
                .padding(padding::top(4.0).bottom(-2.0)),
                space::horizontal()
            ]
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .padding(padding::all(10)),
        )
        .align_x(Alignment::Center)
        .style(|theme| {
            tf2::container::outlined(theme)
                .background(theme.colors().surface.surface_container.lowest)
                .shadow(shadow_from_elevation(elevation(1), theme.colors().shadow))
        });

        let console_output = {
            let console_output_text = column(console.output.iter().map(|text| {
                match text {
                    TextType::Input(string) => iced_selection::text(format!("{}", string))
                        .style(|_theme| iced_selection::text::Style {
                            color: Some(Color::from_rgb8(120, 120, 120)),
                            ..Default::default()
                        })
                        .into(),
                    TextType::Output(string) => iced_selection::text(format!("{}", string)).into(),
                }
            }))
            .padding(5);

            container(
                scrollable(console_output_text)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new().width(15).scroller_width(12),
                    ))
                    .anchor_bottom()
                    .auto_scroll(true)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(padding::left(10))
            .style(|theme| {
                tf2::container::outlined(theme)
                    .background(theme.colors().surface.surface_container.lowest)
                    .shadow(shadow_from_elevation(elevation(1), theme.colors().shadow))
            })
        };

        let console_input =
            textinput_terminal::TextInput::new("Type your command...", &console.input)
                .on_input(Message::ServerTerminalInput)
                .on_submit(Message::SubmitServerTerminalInput)
                .on_key_press(Message::OnKeyPress)
                .width(Length::Fill)
                .line_height(LineHeight::Relative(1.4));

        container(column![header, console_output, console_input].spacing(20))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|theme| tf2::container::surface(theme))
            .into()
    }
}

pub fn find_available_port(ip: Ipv4Addr) -> u16 {
    let socket = UdpSocket::bind((ip, 27015))
        .or_else(|_| UdpSocket::bind((ip, 0)))
        .unwrap();

    socket.local_addr().unwrap().port()
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
