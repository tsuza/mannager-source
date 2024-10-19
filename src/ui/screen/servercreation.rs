use std::path::PathBuf;

use iced::widget::tooltip;
use iced::{color, Color, Font};
use iced::{
    futures::{SinkExt, Stream},
    padding,
    stream::try_channel,
    widget::{
        button, center, column, container, horizontal_rule, progress_bar, row, svg, text,
        text_input,
    },
    Alignment, ContentFit, Element, Length, Subscription, Task,
};
use iced_aw::number_input;
use rfd::FileHandle;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::core;
use crate::core::depotdownloader::DepotDownloader;
use crate::ui::style::{self, icon};

use super::serverlist::{self, get_arg_game_name, SourceAppIDs};

#[derive(Default)]
pub struct State {
    pub form_page: FormPage,
    pub form_info: FormInfo,
}

#[derive(Default, PartialEq, Eq)]
pub enum FormPage {
    #[default]
    GameSelection,
    ServerPath,
    Downloading,
    ServerInfo,
    EditPage,
}

#[derive(Default, Clone)]
pub struct FormInfo {
    pub source_game: SourceAppIDs,
    pub server_name: String,
    pub server_path: PathBuf,
    pub download_output: Vec<String>,
    pub is_downloading: bool,
    pub progress_percent: f32,
    pub map_name: String,
    pub server_description: String,
    pub max_players: u32,
    pub password: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub enum Message {
    GameChosen(SourceAppIDs),
    ServerNameInput(String),
    OpenFilePicker,
    ServerPathChosen(Option<FileHandle>),
    DownloadServer,
    DownloadProgress(Result<Progress, Error>),
    SelectMap,
    ServerMapChosen(Option<FileHandle>),
    MessageDescriptionUpdate(String),
    MaxPlayersUpdate(u32),
    PasswordUpdate(String),
    FinishServerCreation,
    PortUpdate(String),
}

impl State {
    pub fn new() -> Self {
        Self {
            form_info: FormInfo {
                max_players: 24,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn from_server_entry(server_info: &serverlist::ServerInfo) -> Self {
        Self {
            form_info: FormInfo {
                server_name: server_info.name.clone(),
                source_game: server_info.game.clone(),
                server_description: server_info.description.clone(),
                map_name: server_info.map.clone(),
                server_path: server_info.path.clone(),
                max_players: server_info.max_players.clone(),
                password: server_info.password.clone(),
                port: server_info.port.clone(),
                ..Default::default()
            },
            form_page: FormPage::EditPage,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerNameInput(str) => {
                self.form_info.server_name = str;

                Task::none()
            }
            Message::OpenFilePicker => Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Set the server's installation path")
                    .pick_folder(),
                Message::ServerPathChosen,
            ),
            Message::ServerPathChosen(file_handle) => {
                if let Some(file) = file_handle {
                    self.form_info.server_path = file.path().to_path_buf();
                }

                Task::none()
            }
            Message::DownloadServer => {
                self.form_info.is_downloading = true;

                self.form_page = FormPage::Downloading;

                Task::run(
                    download_server(&self.form_info.server_path, &self.form_info.source_game),
                    Message::DownloadProgress,
                )
            }
            Message::DownloadProgress(progress) => {
                if let Ok(progress) = progress {
                    match progress {
                        Progress::Downloading(string) => {
                            if let Some(percent) = string.split("%").next() {
                                if let Ok(percent) = percent.trim().parse::<f32>() {
                                    self.form_info.progress_percent = percent;
                                }
                            }
                        }
                        Progress::Finished => {
                            self.form_info.is_downloading = false;
                            self.form_page = FormPage::ServerInfo;
                        }
                    }
                }

                Task::none()
            }
            Message::GameChosen(source_app_id) => {
                self.form_info.source_game = source_app_id;

                self.form_page = FormPage::ServerPath;

                Task::none()
            }
            Message::SelectMap => Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Choose a default map")
                    .set_directory(format!(
                        "{}/{}/maps",
                        self.form_info.server_path.to_str().unwrap(),
                        get_arg_game_name(self.form_info.source_game.clone())
                    ))
                    .add_filter("Source Map", &["bsp"])
                    .pick_file(),
                Message::ServerMapChosen,
            ),
            Message::ServerMapChosen(file_handle) => {
                if let Some(file) = file_handle {
                    self.form_info.map_name = file
                        .path()
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .and_then(|string| Some(string.to_string()))
                        .unwrap()
                }

                Task::none()
            }
            Message::MessageDescriptionUpdate(description) => {
                self.form_info.server_description = description;

                Task::none()
            }
            Message::MaxPlayersUpdate(number) => {
                self.form_info.max_players = number;

                Task::none()
            }
            Message::PasswordUpdate(password) => {
                self.form_info.password = password;

                Task::none()
            }
            Message::FinishServerCreation => Task::none(),
            Message::PortUpdate(port) => {
                self.form_info.port = if port.is_empty() {
                    0
                } else if let Ok(port) = port.parse::<u16>() {
                    port
                } else {
                    self.form_info.port
                };

                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        match self.form_page {
            FormPage::GameSelection => choose_game_container(),
            FormPage::ServerPath => server_creation_form_container(&self.form_info),
            FormPage::Downloading => downloading_container(&self.form_info),
            FormPage::ServerInfo => server_creation_info(&self.form_info),
            FormPage::EditPage => edit_server_info(&self.form_info),
        }
    }
}

fn choose_game_container<'a>() -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let game_entry = |game_name: &'static str, image_path: svg::Handle, button_event: Message| {
        tooltip(
            button(svg(image_path).width(60).content_fit(ContentFit::Contain))
                .on_press(button_event)
                .padding(0)
                .style(|_theme, _status| button::Style {
                    background: None,
                    ..button::Style::default()
                }),
            container(game_name).padding(10),
            tooltip::Position::Top,
        )
    };

    container(column![
        text!("Server creation")
            .font(Font::with_name("TF2 Build"))
            .size(32)
            .color(Color::WHITE)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        horizontal_rule(0),
        text!("Select the game server").size(20).color(Color::WHITE),
        container(
            row![
                game_entry(
                    "Team Fortress 2",
                    svg::Handle::from_path("images/tf2-logo.svg"),
                    Message::GameChosen(SourceAppIDs::TeamFortress2)
                ),
                game_entry(
                    "Counter Strike: Source",
                    svg::Handle::from_path("images/css-logo.svg"),
                    Message::GameChosen(SourceAppIDs::CounterStrikeSource)
                ),
                game_entry(
                    "Counter Strike 2",
                    svg::Handle::from_path("images/cs2-logo.svg"),
                    Message::GameChosen(SourceAppIDs::CounterStrike2)
                ),
                game_entry(
                    "Left For Dead 1",
                    svg::Handle::from_path("images/l4d1-logo.svg"),
                    Message::GameChosen(SourceAppIDs::LeftForDead1)
                ),
                game_entry(
                    "Left For Dead 2",
                    svg::Handle::from_path("images/l4d2-logo.svg"),
                    Message::GameChosen(SourceAppIDs::LeftForDead2)
                ),
                game_entry(
                    "Half Life 2: Deathmatch",
                    svg::Handle::from_path("images/hl2mp-logo.svg"),
                    Message::GameChosen(SourceAppIDs::HalfLife2DM)
                ),
                game_entry(
                    "No More Room In Hell",
                    svg::Handle::from_path("images/nmrih-logo.svg"),
                    Message::GameChosen(SourceAppIDs::NoMoreRoomInHell)
                ),
            ]
            .spacing(20)
            .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .padding(50)
    ])
    .width(720)
    .padding(10)
    .style(|_theme| style::tf2::Style::primary_container(_theme))
    .into()
}

fn server_creation_form_container<'a>(state: &FormInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(
        column![
            column![
                text!("Server creation")
                    .font(Font::with_name("TF2 Build"))
                    .size(32)
                    .color(Color::WHITE)
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
                horizontal_rule(0),
            ],
            row![
                text!("Server Name")
                    .color(Color::WHITE)
                    .width(Length::FillPortion(1)),
                text_input("server name", state.server_name.as_str())
                    .on_input(Message::ServerNameInput)
                    .width(Length::FillPortion(2))
                    .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
            ]
            .align_y(Alignment::Center),
            row![
                text!("Server Path")
                    .color(Color::WHITE)
                    .width(Length::FillPortion(1)),
                container(
                    button("Click to pick a directory")
                        .on_press(Message::OpenFilePicker)
                        .style(|_theme, _status| style::tf2::Style::form_button(_theme, _status))
                )
                .width(Length::FillPortion(2))
                .align_x(Alignment::Center)
            ]
            .align_y(Alignment::Center),
            container(
                button(text!("Create").size(25))
                    .on_press(Message::DownloadServer)
                    .style(|_theme, _status| style::tf2::Style::button(_theme, _status))
            )
            .width(Length::Fill)
            .padding(padding::top(50))
            .align_x(Alignment::Center)
        ]
        .spacing(20),
    )
    .padding(10)
    .width(720)
    .height(600)
    .height(Length::Shrink)
    .style(|_theme| style::tf2::Style::primary_container(_theme))
    .into()
}

fn downloading_container<'a>(state: &FormInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(column![
        text!("Downloading the server...")
            .font(Font::with_name("TF2 Build"))
            .size(32)
            .color(Color::WHITE)
            .width(Length::Fill)
            .align_x(Alignment::Center),
        horizontal_rule(0),
        center(
            progress_bar(0.0..=100.0, state.progress_percent)
                .height(20)
                .width(300)
        )
        .width(Length::Fill)
        .height(Length::Fill)
    ])
    .width(720)
    .height(400)
    .padding(10)
    .style(|_theme| style::tf2::Style::primary_container(_theme))
    .into()
}

fn server_creation_info<'a>(state: &FormInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let port = if state.port != 0 {
        &state.port.to_string()
    } else {
        ""
    };

    container(
        column![
            text!("Server creation")
                .font(Font::with_name("TF2 Build"))
                .size(32)
                .color(Color::WHITE)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            horizontal_rule(0),
            column![
                row![
                    text!("Server Description")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    text_input("Server Description", &state.server_description)
                        .on_input(Message::MessageDescriptionUpdate)
                        .width(Length::FillPortion(2))
                        .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
                ]
                .align_y(Alignment::Center),
                row![
                    text!("Map")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    container(
                        button("Select Map").on_press(Message::SelectMap).style(
                            |_theme, _status| style::tf2::Style::form_button(_theme, _status)
                        )
                    )
                    .width(Length::FillPortion(2))
                ]
                .align_y(Alignment::Center),
                row![
                    text!("Max Players")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    container(number_input(
                        state.max_players,
                        0..=100,
                        Message::MaxPlayersUpdate
                    ))
                    .width(Length::FillPortion(2))
                ]
                .align_y(Alignment::Center),
                row![
                    text!("Server Password")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    text_input("Server Password", &state.password)
                        .on_input(Message::PasswordUpdate)
                        .width(Length::FillPortion(2))
                        .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
                ]
                .align_y(Alignment::Center),
                row![
                    row![
                        text!("Port").color(Color::WHITE),
                        tooltip(
                            icon::warning().color(color!(0xeee5cf)),
                            text!("If it's left empty, the app will automatically find an available port.").width(350),
                            tooltip::Position::Top
                        )
                        .gap(10)
                        .padding(20)
                        .style(|_theme| style::tf2::Style::tooltip_container(_theme))
                    ]
                    .spacing(10)
                    .width(Length::FillPortion(1)),
                    text_input("Port", port)
                        .on_input(Message::PortUpdate)
                        .width(Length::FillPortion(2))
                        .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
                ]
                .align_y(Alignment::Center),
                container(
                    button(text!("Finish").size(20))
                        .on_press(Message::FinishServerCreation)
                        .style(|_theme, _status| style::tf2::Style::button(_theme, _status))
                )
                .width(Length::Fill)
                .align_x(Alignment::Center)
            ]
            .spacing(15)
            .padding(padding::top(10))
        ]
        .spacing(5),
    )
    .padding(10)
    .width(720)
    .height(400)
    .style(|_theme| style::tf2::Style::primary_container(_theme))
    .into()
}

fn edit_server_info<'a>(state: &FormInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let port = if state.port != 0 {
        &state.port.to_string()
    } else {
        ""
    };

    container(
        column![
            text!("Edit server")
                .font(Font::with_name("TF2 Build"))
                .size(32)
                .color(Color::WHITE)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            horizontal_rule(0),
            column![
                row![
                    text!("Name")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    text_input("name...", &state.server_name)
                        .on_input(Message::ServerNameInput)
                        .width(Length::FillPortion(2))
                        .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
                ]
                .align_y(Alignment::Center),
                row![
                    text!("Server Description")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    text_input("Server Description", &state.server_description)
                        .on_input(Message::MessageDescriptionUpdate)
                        .width(Length::FillPortion(2))
                        .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
                ]
                .align_y(Alignment::Center),
                row![
                    text!("Map")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    container(
                        button("Select Map").on_press(Message::SelectMap).style(
                            |_theme, _status| style::tf2::Style::form_button(_theme, _status)
                        )
                    )
                    .width(Length::FillPortion(2))
                ]
                .align_y(Alignment::Center),
                row![
                    text!("Max Players")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    container(number_input(
                        state.max_players,
                        0..=100,
                        Message::MaxPlayersUpdate
                    ))
                    .width(Length::FillPortion(2))
                ]
                .align_y(Alignment::Center),
                row![
                    text!("Server Password")
                        .color(Color::WHITE)
                        .width(Length::FillPortion(1)),
                    text_input("Server Password", &state.password)
                        .on_input(Message::PasswordUpdate)
                        .width(Length::FillPortion(2))
                        .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
                ]
                .align_y(Alignment::Center),
                row![
                    row![
                        text!("Port").color(Color::WHITE),
                        tooltip(
                            icon::warning().color(color!(0xeee5cf)),
                            text!("If it's left empty, the app will automatically find an available port.").width(350),
                            tooltip::Position::Top
                        )
                        .gap(10)
                        .padding(20)
                        .style(|_theme| style::tf2::Style::tooltip_container(_theme))
                    ]
                    .spacing(10)
                    .width(Length::FillPortion(1)),
                    text_input("Port", port)
                        .on_input(Message::PortUpdate)
                        .width(Length::FillPortion(2))
                        .style(|_theme, _status| style::tf2::Style::text_input(_theme, _status))
                ]
                .align_y(Alignment::Center),
                container(
                    button(text!("Finish").size(20))
                        .on_press(Message::FinishServerCreation)
                        .style(|_theme, _status| style::tf2::Style::button(_theme, _status))
                )
                .width(Length::Fill)
                .align_x(Alignment::Center)
            ]
            .spacing(15)
            .padding(padding::top(10))
        ]
        .spacing(5),
    )
    .padding(10)
    .width(720)
    .height(400)
    .style(|_theme| style::tf2::Style::primary_container(_theme))
    .into()
}

pub fn download_server(
    path: &PathBuf,
    appid: &SourceAppIDs,
) -> impl Stream<Item = Result<Progress, Error>> {
    let testun = path
        .to_str()
        .and_then(|string| Some(string.to_string()))
        .unwrap_or("server".to_string());

    let appid = appid.clone();

    try_channel(1, move |mut output| async move {
        let mut depot = DepotDownloader::new("./depotdownloader").await?;

        let stdout = depot.download_app(&testun, appid.into()).await?;

        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();

            while let Some(line) = reader
                .next_line()
                .await
                .map_err(|err| Error::Io(err.to_string()))?
            {
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

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("")]
    ServerDownloadError(#[from] core::Error),

    #[error("Io error: {0}")]
    Io(String),
}
