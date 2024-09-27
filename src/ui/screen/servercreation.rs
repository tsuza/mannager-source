use core::error;
use std::path::PathBuf;

use iced::widget::{scrollable, tooltip};
use iced::Font;
use iced::{
    border, color,
    futures::{SinkExt, Stream, StreamExt},
    padding,
    stream::try_channel,
    widget::{
        button, center, column, container, horizontal_rule, progress_bar, row,
        rule::{self, FillMode},
        scrollable::Viewport,
        stack, svg, text, text_input, vertical_rule, vertical_space,
    },
    Alignment, Background, ContentFit, Element, Error, Length, Padding, Shadow, Subscription, Task,
};
use iced_aw::number_input;
use rfd::FileHandle;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
};

use crate::core::depotdownloader::DepotDownloader;

use super::serverlist::SourceAppIDs;

#[derive(Default)]
pub struct State {
    pub form_page: FormPage,
    pub form_info: FormInfo,
}

#[derive(Default)]
pub enum FormPage {
    #[default]
    GameSelection,
    ServerPath,
    Downloading,
    ServerInfo,
}

#[derive(Default)]
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
    OnDownloadingScrollableScroll(Viewport),
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
            Message::OnDownloadingScrollableScroll(viewport) => Task::none(),
            Message::SelectMap => Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Choose a default map")
                    .set_directory(format!(
                        "{}/tf/maps",
                        self.form_info.server_path.to_str().unwrap()
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
        container(
            text!("Select a game server")
                .font(Font::with_name("TF2 Build"))
                .size(32)
                .color(color!(0xFFF))
        )
        .padding(10),
        container(
            row![
                tooltip(
                    button(
                        svg(svg::Handle::from_path("images/tf2-logo.svg"))
                            .width(128)
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
                        svg(svg::Handle::from_path("images/cs2-logo.svg"))
                            .width(128)
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
                            .width(128)
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
                            .width(128)
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
            ]
            .spacing(20)
            .align_y(Alignment::Center)
        )
        .padding(50)
    ])
    .padding(10)
    .style(|_theme| container::background(color!(0x34302d)))
    .into()
}

fn server_creation_form_container<'a>(state: &FormInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(column![
        row![
            "Server Name",
            text_input("server name", state.server_name.as_str())
                .on_input(Message::ServerNameInput)
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
        container(button("Create").on_press(Message::DownloadServer)).align_x(Alignment::Center)
    ])
    .padding(10)
    .width(600)
    .height(600)
    .height(Length::Shrink)
    .style(|_theme| container::background(color!(0x34302d)).border(border::rounded(5)))
    .into()
}

fn downloading_container<'a>(state: &FormInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        container(scrollable(""))
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

fn server_creation_info<'a>(state: &FormInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(column![
        text_input("Server Description", &state.server_description)
            .on_input(Message::MessageDescriptionUpdate),
        button("Select Map").on_press(Message::SelectMap),
        number_input(state.max_players, 0..=100, Message::MaxPlayersUpdate),
        text_input("Server Password", &state.password).on_input(Message::PasswordUpdate),
        button("Finish").on_press(Message::FinishServerCreation)
    ])
    .padding(10)
    .width(720)
    .height(400)
    .style(|_theme| container::background(color!(0x34302d)).border(border::rounded(5)))
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
