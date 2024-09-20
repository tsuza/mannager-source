use core::error;
use std::path::PathBuf;

use iced::{
    border, color,
    futures::{SinkExt, Stream, StreamExt},
    padding,
    stream::try_channel,
    widget::{
        button, center, column, container, horizontal_rule, progress_bar, row,
        rule::{self, FillMode},
        scrollable, stack, svg, text, text_input, vertical_rule, vertical_space,
    },
    Alignment, Background, ContentFit, Element, Error, Length, Padding, Shadow, Subscription, Task,
};
use rfd::FileHandle;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
};

use crate::{
    core::depotdownloader::{self, DepotDownloader},
    ui::components::modal::modal,
};

pub struct ServerList {
    is_create_server_popup_visible: bool,
    is_add_info_to_server_creation: bool,
    server_name: String,
    server_path: PathBuf,
    download_output: Vec<String>,
    is_downloading: bool,
    progress_percent: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateServer,
    OnClickOutsidePopup,
    OnGameChosen(SourceAppIDs),
    OnServerName(String),
    OpenFilePicker,
    ServerPathChosen(Option<FileHandle>),
    DownloadServer,
    DownloadProgress(Result<Progress, CustomError>),
}

#[derive(Debug, Clone)]
pub enum SourceAppIDs {
    TeamFortress2,
    CounterStrike2,
    LeftForDead1,
    LeftForDead2,
}

impl From<u32> for SourceAppIDs {
    fn from(value: u32) -> Self {
        match value {
            232250 => SourceAppIDs::TeamFortress2,
            730 => SourceAppIDs::CounterStrike2,
            222840 => SourceAppIDs::LeftForDead1,
            222860 => SourceAppIDs::LeftForDead2,
            _ => panic!("Unsupported App ID"),
        }
    }
}

impl ServerList {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                is_create_server_popup_visible: false,
                is_add_info_to_server_creation: false,
                server_name: "Test".into(),
                server_path: PathBuf::new(),
                download_output: vec!["".to_string()],
                is_downloading: false,
                progress_percent: 0.0,
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
                self.is_create_server_popup_visible = !self.is_create_server_popup_visible;

                Task::none()
            }
            Message::OnClickOutsidePopup => {
                self.is_create_server_popup_visible = false;

                Task::none()
            }
            Message::OnGameChosen(x) => {
                self.is_add_info_to_server_creation = true;

                Task::none()
            }
            Message::OnServerName(str) => {
                self.server_name = str;

                Task::none()
            }
            Message::OpenFilePicker => Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Set the server's installation path")
                    .set_directory("./")
                    .pick_folder(),
                Message::ServerPathChosen,
            ),
            Message::ServerPathChosen(file_handle) => {
                if let Some(file) = file_handle {
                    self.server_path = file.path().into();
                }

                Task::none()
            }
            Message::DownloadServer => {
                self.is_downloading = true;

                Task::none()
            }
            Message::DownloadProgress(progress) => {
                if let Ok(progress) = progress {
                    match progress {
                        Progress::Downloading(string) => {
                            self.download_output.push(string.clone());

                            if let Some(percent) = string.split("%").next() {
                                if let Ok(percent) = percent.trim().parse::<f32>() {
                                    self.progress_percent = percent;
                                }
                            }
                        }
                        Progress::Finished => (),
                    }
                }

                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.is_downloading {
            Subscription::run_with_id(1, download_server(&self.server_path, 232250u32))
                .map(Message::DownloadProgress)
        } else {
            Subscription::none()
        }
    }

    pub fn view(&self) -> Element<Message> {
        let base = container(column![
            navbar(),
            container(show_servers()).padding(padding::all(10).top(30))
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::background(color!(0x34302d)));

        if self.is_create_server_popup_visible {
            modal(
                base,
                if !self.is_downloading {
                    if self.is_add_info_to_server_creation {
                        create_server_container_step_two(self)
                    } else {
                        create_server_container()
                    }
                } else {
                    downloading_container(self)
                },
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
            color: color!(0x6a4535),
            width: 3,
            fill_mode: FillMode::Full,
            ..rule::default(_theme)
        })
    ])
    .width(Length::Fill)
    .height(64)
    .padding(0)
    .style(|_theme| container::background(color!(0x462d26)))
    .into()
}

fn show_servers<'a>() -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    column![
        container("Server name")
            .width(Length::Fill)
            .max_width(720)
            .height(128)
            .padding(10)
            .style(|_theme| container::background(color!(0x462d26))
                .border(border::color(color!(0x6a4535)).rounded(10).width(3))),
        container("Server name")
            .width(Length::Fill)
            .max_width(720)
            .height(128)
            .padding(10)
            .style(|_theme| container::background(color!(0x462d26))
                .border(border::color(color!(0x6a4535)).rounded(10).width(3))),
        container("Server name")
            .width(Length::Fill)
            .max_width(720)
            .height(128)
            .padding(10)
            .style(|_theme| container::background(color!(0x462d26))
                .border(border::color(color!(0x6a4535)).rounded(10).width(3))),
        container("Server name")
            .width(Length::Fill)
            .max_width(720)
            .height(128)
            .padding(10)
            .style(|_theme| container::background(color!(0x462d26))
                .border(border::color(color!(0x6a4535)).rounded(10).width(3))),
        button("+").on_press(Message::CreateServer)
    ]
    .align_x(Alignment::Center)
    .spacing(10)
    .into()
}

fn create_server_container<'a>() -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(column![
        text!("Create a server").size(32).color(color!(0xFFF)),
        container(
            row![
                button(
                    svg(svg::Handle::from_path("images/tf2-logo.svg"))
                        .width(128)
                        .content_fit(ContentFit::Contain)
                )
                .on_press(Message::OnGameChosen(SourceAppIDs::TeamFortress2))
                .padding(0)
                .style(|_theme, _status| button::Style {
                    background: None,
                    ..button::Style::default()
                }),
                button(
                    svg(svg::Handle::from_path("images/cs2-logo.svg"))
                        .width(128)
                        .content_fit(ContentFit::Contain)
                )
                .on_press(Message::OnGameChosen(SourceAppIDs::CounterStrike2))
                .padding(0)
                .style(|_theme, _status| button::Style {
                    background: None,
                    ..button::Style::default()
                }),
                button(
                    svg(svg::Handle::from_path("images/l4d1-logo.svg"))
                        .width(128)
                        .content_fit(ContentFit::Contain)
                )
                .on_press(Message::OnGameChosen(SourceAppIDs::LeftForDead1))
                .padding(0)
                .style(|_theme, _status| button::Style {
                    background: None,
                    ..button::Style::default()
                }),
                button(
                    svg(svg::Handle::from_path("images/l4d2-logo.svg"))
                        .width(128)
                        .content_fit(ContentFit::Contain)
                )
                .on_press(Message::OnGameChosen(SourceAppIDs::LeftForDead2))
                .padding(0)
                .style(|_theme, _status| button::Style {
                    background: None,
                    ..button::Style::default()
                }),
            ]
            .align_y(Alignment::Center)
        )
        .padding(50)
    ])
    .padding(10)
    .style(|_theme| container::background(color!(0x34302d)))
    .into()
}

fn create_server_container_step_two<'a>(state: &ServerList) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(column![
        row![
            "Server Name",
            text_input("server name", state.server_name.as_str())
                .on_input(Message::OnServerName)
                .width(350)
        ]
        .align_y(Alignment::Center)
        .spacing(50),
        row![
            "Server Path",
            button("Pick a directory").on_press(Message::OpenFilePicker)
        ]
        .align_y(Alignment::Center)
        .spacing(50),
        center(button("Create").on_press(Message::DownloadServer))
    ])
    .padding(10)
    .style(|_theme| container::background(color!(0x34302d)).border(border::rounded(5)))
    .into()
}

fn downloading_container<'a>(state: &ServerList) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        container(
            scrollable(column(state.download_output.iter().map(|element| {
                text!("{}", element)
                    .wrapping(text::Wrapping::None)
                    .style(|_theme| text::Style {
                        color: Some(color!(0, 0, 0, 0.1)),
                    })
                    .into()
            })))
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new().width(0).scroller_width(0),
            ))
            .anchor_bottom()
        )
        .padding(10)
        .width(720)
        .height(400)
        .style(|_theme| container::background(color!(0x34302d)).border(border::rounded(5))),
        center(
            progress_bar(0.0..=100.0, state.progress_percent)
                .height(10)
                .width(300)
        )
        .width(Length::Fill)
        .height(Length::Fill),
    ]
    .into()
}

fn download_server(
    path: &PathBuf,
    appid: impl Into<u32>,
) -> impl Stream<Item = Result<Progress, CustomError>> {
    let testun = path.to_str().unwrap().to_string();

    try_channel(1, move |mut output| async move {
        let mut depot = DepotDownloader::new("./depotdownloader").await.unwrap();

        let stdout = depot.download_app(&testun, appid.into()).await.unwrap();

        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();

            while let Some(line) = reader.next_line().await.unwrap() {
                let _ = output.send(Progress::Downloading(line)).await;
            }
        }

        let _ = output.send(Progress::Finished).await;

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum Progress {
    Downloading(String),
    Finished,
}

#[derive(Debug, Clone)]
pub enum CustomError {
    Null,
}
