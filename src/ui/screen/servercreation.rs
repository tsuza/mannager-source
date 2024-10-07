use std::path::PathBuf;

use iced::widget::tooltip;
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
use iced::{Color, Font};
use iced_aw::number_input;
use rfd::FileHandle;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::core::depotdownloader::DepotDownloader;
use crate::ui::style;

use super::serverlist::{get_arg_game_name, SourceAppIDs};

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
}

#[derive(Debug, Clone)]
pub enum Message {
    GameChosen(SourceAppIDs),
    ServerNameInput(String),
    OpenFilePicker,
    ServerPathChosen(Option<FileHandle>),
    DownloadServer,
    DownloadProgress(Result<Progress, CustomError>),
    SelectMap,
    ServerMapChosen(Option<FileHandle>),
    MessageDescriptionUpdate(String),
    MaxPlayersUpdate(u32),
    PasswordUpdate(String),
    FinishServerCreation,
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
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string();
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
        }
    }
}

fn choose_game_container<'a>() -> Element<'a, Message>
where
    Message: Clone + 'a,
{
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
                tooltip(
                    button(
                        svg(svg::Handle::from_path("images/tf2-logo.svg"))
                            .width(60)
                            .content_fit(ContentFit::Contain)
                    )
                    .on_press(Message::GameChosen(SourceAppIDs::TeamFortress2))
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        ..button::Style::default()
                    }),
                    container("Team Fortress 2").padding(10),
                    tooltip::Position::Top
                ),
                tooltip(
                    button(
                        svg(svg::Handle::from_path("images/css-logo.svg"))
                            .width(60)
                            .content_fit(ContentFit::Contain)
                    )
                    .on_press(Message::GameChosen(SourceAppIDs::CounterStrikeSource))
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        ..button::Style::default()
                    }),
                    container("Counter Strike: Source").padding(10),
                    tooltip::Position::Top
                ),
                tooltip(
                    button(
                        svg(svg::Handle::from_path("images/cs2-logo.svg"))
                            .width(60)
                            .content_fit(ContentFit::Contain)
                    )
                    .on_press(Message::GameChosen(SourceAppIDs::CounterStrike2))
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        ..button::Style::default()
                    }),
                    container("Counter Strike 2").padding(10),
                    tooltip::Position::Top
                ),
                tooltip(
                    button(
                        svg(svg::Handle::from_path("images/l4d1-logo.svg"))
                            .width(60)
                            .content_fit(ContentFit::Contain)
                    )
                    .on_press(Message::GameChosen(SourceAppIDs::LeftForDead1))
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        ..button::Style::default()
                    }),
                    container("Left For Dead 1").padding(10),
                    tooltip::Position::Top
                ),
                tooltip(
                    button(
                        svg(svg::Handle::from_path("images/l4d2-logo.svg"))
                            .width(60)
                            .content_fit(ContentFit::Contain)
                    )
                    .on_press(Message::GameChosen(SourceAppIDs::LeftForDead2))
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        ..button::Style::default()
                    }),
                    container("Left For Dead 2").padding(10),
                    tooltip::Position::Top
                ),
                tooltip(
                    button(
                        svg(svg::Handle::from_path("images/hl2mp-logo.svg"))
                            .width(60)
                            .content_fit(ContentFit::Contain)
                    )
                    .on_press(Message::GameChosen(SourceAppIDs::HalfLife2DM))
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        ..button::Style::default()
                    }),
                    container("Half Life 2: Deathmatch").padding(10),
                    tooltip::Position::Top
                ),
                tooltip(
                    button(
                        svg(svg::Handle::from_path("images/nmrih-logo.svg"))
                            .width(60)
                            .content_fit(ContentFit::Contain)
                    )
                    .on_press(Message::GameChosen(SourceAppIDs::NoMoreRoomInHell))
                    .padding(0)
                    .style(|_theme, _status| button::Style {
                        background: None,
                        ..button::Style::default()
                    }),
                    container("No More Room In Hell").padding(10),
                    tooltip::Position::Top
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

fn download_server(
    path: &PathBuf,
    appid: &SourceAppIDs,
) -> impl Stream<Item = Result<Progress, CustomError>> {
    let testun = path.to_str().unwrap().to_string();
    let appid = appid.clone();

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
