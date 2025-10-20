use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::ui::Element;
use crate::ui::components::metered_progress_bar;
use crate::ui::components::notification::notification;
use crate::ui::games::{SOURCE_GAMES, SourceGame};
use crate::ui::icons::{close, warning};
use crate::ui::server::ServerInfo;
use crate::ui::themes::tf2;
use iced::Font;
use iced::widget::{Row, float, rule, space, tooltip};
use iced::{
    Alignment, ContentFit, Length, Task, padding,
    task::{Straw, sipper},
    widget::{button, center, column, container, row, svg, text, text_input},
};
use iced_aw::number_input;
use rfd::FileHandle;
use snafu::{ResultExt, Snafu};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::core::depotdownloader::DepotDownloader;
use crate::core::{self, Game, get_arg_game_name};

pub struct State {
    form_page: FormSection,
    server: ServerInfo,
    is_downloading: bool,
    progress: f32,
}

#[derive(Debug)]
pub enum Action {
    None,
    SwitchToServerList,
    ServerCreated(ServerInfo),
    Run(Task<Message>),
}

#[derive(Default, PartialEq, Eq)]
pub enum FormSection {
    #[default]
    GameSelection,
    Downloading,
    ServerInfo,
}

#[derive(Debug, Clone)]
pub enum Message {
    GameChosen(Game),
    ServerNameInput(String),
    ChooseServerPath,
    ChooseServerPathFinished(Option<FileHandle>),
    DownloadServer,
    Downloading(Update),
    SelectMap,
    SelectMapFinished(Option<FileHandle>),
    MessageDescriptionUpdate(String),
    MaxPlayersUpdate(u32),
    PasswordUpdate(String),
    FinishServerCreation,
    PortUpdate(String),
    GsltUpdate(String),
    CloseServerCreation,
}

impl State {
    pub fn new() -> Self {
        Self {
            server: ServerInfo {
                max_players: 24,
                ..Default::default()
            },
            ..Self::default()
        }
    }

    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::ServerNameInput(str) => {
                self.server.name = str;

                Action::None
            }
            Message::ChooseServerPath => Action::Run(Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Set the server's installation path")
                    .pick_folder(),
                Message::ChooseServerPathFinished,
            )),
            Message::ChooseServerPathFinished(file_handle) => {
                if let Some(file) = file_handle {
                    self.server.path = file.path().to_path_buf();
                }

                Action::None
            }
            Message::DownloadServer => {
                self.is_downloading = true;

                self.form_page = FormSection::Downloading;

                let server_path = self.server.path.clone();
                let source_game = self.server.game.clone();

                Action::Run(
                    Task::sip(
                        download_server(server_path, source_game),
                        Update::Downloading,
                        Update::Finished,
                    )
                    .map(Message::Downloading),
                )
            }
            Message::Downloading(progress) => match progress {
                Update::Downloading(percent) => {
                    self.progress = percent;

                    Action::None
                }
                Update::Finished(_) => {
                    self.is_downloading = false;
                    self.form_page = FormSection::ServerInfo;

                    Action::Run(
                        Task::future(notification(
                            "MANNager",
                            "The server has finished downloading",
                            Duration::from_secs(5),
                        ))
                        .discard(),
                    )
                }
            },
            Message::GameChosen(source_app_id) => {
                self.server.game = source_app_id;

                Action::None
            }
            Message::SelectMap => Action::Run(Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Choose a default map")
                    .set_directory(format!(
                        "{}/{}/maps",
                        self.server.path.display().to_string(),
                        get_arg_game_name(&self.server.game.clone())
                    ))
                    .add_filter("Source Map", &["bsp", "vpk"])
                    .pick_file(),
                Message::SelectMapFinished,
            )),
            Message::SelectMapFinished(file_handle) => {
                if let Some(file) = file_handle {
                    self.server.map = file
                        .path()
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .and_then(|string| Some(string.to_string()))
                        .unwrap()
                }

                Action::None
            }
            Message::MessageDescriptionUpdate(description) => {
                self.server.description = Some(description);

                Action::None
            }
            Message::MaxPlayersUpdate(number) => {
                self.server.max_players = number;

                Action::None
            }
            Message::PasswordUpdate(password) => {
                self.server.password = (!password.is_empty()).then_some(password);

                Action::None
            }
            Message::FinishServerCreation => {
                let server = std::mem::replace(&mut self.server, ServerInfo::default());

                Action::ServerCreated(server)
            }
            Message::PortUpdate(port) => {
                self.server.port = (!port.is_empty())
                    .then(|| port.parse::<u16>().ok())
                    .flatten();

                Action::None
            }
            Message::GsltUpdate(token) => {
                self.server.gslt = (!token.is_empty()).then_some(token);

                Action::None
            }
            Message::CloseServerCreation => Action::SwitchToServerList,
        }
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        match self.form_page {
            FormSection::GameSelection => choose_game_container(&self.server),
            FormSection::Downloading => downloading_container(self.progress),
            FormSection::ServerInfo => server_creation_info(&self.server),
        }
    }
}

fn choose_game_container<'a>(server: &ServerInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let game_entry = |game: &SourceGame, button_event: Message| -> Element<'a, Message> {
        if server.game == game.game {
            Element::from(
                float(
                    button(
                        svg(game.image.clone())
                            .content_fit(ContentFit::Contain)
                            .height(80)
                            .width(80),
                    )
                    .on_press(button_event)
                    .padding(0)
                    .style(|_theme, _status| tf2::button::text(_theme, _status)),
                )
                .scale(1.2),
            )
        } else {
            Element::from(
                tooltip(
                    button(
                        svg(game.image.clone())
                            .content_fit(ContentFit::Contain)
                            .height(80)
                            .width(80)
                            .opacity(0.5),
                    )
                    .on_press(button_event)
                    .padding(0)
                    .style(|_theme, _status| tf2::button::text(_theme, _status)),
                    container(text!("{}", game.game.to_string())).padding(10),
                    tooltip::Position::Bottom,
                )
                .gap(10)
                .padding(10)
                .style(|theme| tf2::container::tooltip(theme)),
            )
        }
    };

    Element::from(container(
        container(column![
            column![
                row![
                    space::horizontal().width(Length::FillPortion(1)),
                    container(
                    text!("Server creation")
                        .font(Font::with_name("TF2 Build"))
                        .line_height(1.0)
                        .size(40)
                        .align_x(Alignment::Center).align_y(Alignment::Center)).padding(padding::top(3.0).bottom(-3.0)).width(Length::FillPortion(2)).align_x(Alignment::Center),
                    container(button(
                        close()
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .size(20)
                            .center()
                    )
                    .on_press(Message::CloseServerCreation)
                    .width(32)
                    .height(32)).align_right(Length::FillPortion(1))
                ]
                .align_y(Alignment::Center)
                .padding(padding::all(10).bottom(0)),
                container(rule::horizontal(3)).width(200),
                text!("Select the game server").size(25)
            ]
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .spacing(5),
            container(
                column![
                    column![
                        text!("Server Name"),
                        text_input("server name", &server.name)
                            .on_input(Message::ServerNameInput)
                            .width(300)
                            .padding([5, 10])
                    ].spacing(5),
                    column![
                        text!("Server Path"),
                        row![
                            button("Click to pick a directory").on_press(Message::ChooseServerPath),
                            {
                                let path = server.path.display().to_string();

                                (!path.is_empty()).then_some(
                                    container(text(path)).padding(6).style(|theme| {
                                        tf2::container::surface_container_low(theme)
                                    }),
                                )
                            }
                        ]
                        .align_y(Alignment::Center)
                        .spacing(10)
                    ].spacing(5),
                    container(column![
                        row![
                            text!("Server Game"),
                            tooltip(
                                warning(),
                                "Is your game missing? Feel free to open an issue on Github so it can be added!",
                                tooltip::Position::Top
                            )
                            .gap(10)
                            .padding(20)
                            .style(|_theme| tf2::container::tooltip(_theme))
                        ]
                        .spacing(5)
                        .align_y(Alignment::Center),
                        container(
                            Row::from_vec(
                                SOURCE_GAMES
                                    .iter()
                                    .map(|game| game_entry(game, Message::GameChosen(game.game)))
                                    .collect()
                            )
                            .spacing(20)
                            .align_y(Alignment::Center)
                            .wrap()
                            .align_x(Alignment::Center)
                        )
                        .center_x(Length::Fill)
                        .padding(20)
                        .style(|theme| tf2::container::outlined(theme)),
                        container(
                            button(text!("Create").size(20)).on_press(Message::DownloadServer)
                        )
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                    ].spacing(10))
                    .center_x(Length::Fill)
                ]
                .spacing(30)
            )
            .padding(padding::all(50).top(0))
        ])
        .width(1000)
        .padding(padding::all(10))
        .height(Length::Fill)
        .style(|_theme| tf2::container::main(_theme)),
    )
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(40)
    .style(|theme| tf2::container::surface(theme))) //.explain(Color::BLACK)
}

fn downloading_container<'a>(progress: f32) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(
        container(column![
            text!("Downloading the server...")
                .font(Font::with_name("TF2 Build"))
                .size(32)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            rule::horizontal(0),
            center(
                metered_progress_bar(0.0..=100.0, progress)
                    .bars(20)
                    .spacing(4)
                    .length(500)
            )
            .width(Length::Fill)
            .height(Length::Fill)
        ])
        .width(1000)
        .padding(padding::all(50).top(10))
        .height(Length::Fill)
        .style(|_theme| tf2::container::main(_theme)),
    )
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(40)
    .style(|theme| tf2::container::surface(theme))
    .into()
}

fn server_creation_info<'a>(server: &ServerInfo) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    container(
        container(column![
            text!("Server creation")
                .font(Font::with_name("TF2 Build"))
                .line_height(1.0)
                .size(40)
                .width(Length::Fill)
                .align_x(Alignment::Center),
            rule::horizontal(0),
            column![
                column![
                    text!("Server Description").width(Length::FillPortion(1)),
                    text_input(
                        "Server Description",
                        &server.description.as_deref().unwrap_or_default()
                    )
                    .on_input(Message::MessageDescriptionUpdate)
                    .padding([5, 10])

                ]
                .align_x(Alignment::Center).spacing(5),
                column![
                    text!("Map"),
                    row![
                        container(button("Select Map").on_press(Message::SelectMap)),
                        (!server.map.is_empty()).then_some(
                            container(text(server.map.clone())).padding(6).style(|theme| tf2::container::surface_container_low(theme))
                        )
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                ].spacing(5),
                column![
                    text!("Max Players"),
                    container(number_input(
                        &server.max_players,
                        0..=100,
                        Message::MaxPlayersUpdate
                    ).padding([5, 10]))
                ].spacing(5),
                row![
                    column![
                        text!("Server Password"),
                        text_input("Server Password", &server.password.as_deref().unwrap_or_default())
                            .on_input(Message::PasswordUpdate)
                            .secure(true)
                            .width(250)
                            .padding([5, 10])

                    ]
                    .spacing(5),
                    column![
                        row![
                            text!("Port"),
                            tooltip(
                                warning(),
                                text!("If it's left empty, the app will automatically find an available port.").width(350),
                                tooltip::Position::Top
                            )
                            .gap(10)
                            .padding(20)
                            .style(|_theme| tf2::container::tooltip(_theme))
                        ]
                        .spacing(10),
                        text_input("Port", &server.port.map(|port| port.to_string()).unwrap_or_default())
                            .on_input(Message::PortUpdate)
                            .width(70)
                            .padding([5, 10])

                    ]
                    .spacing(5),
                ]
                .align_y(Alignment::Center)
                .spacing(20),
                column![
                    text!("GSLT"),
                    text_input("GSLT", &server.gslt.as_deref().unwrap_or_default())
                        .on_input(Message::GsltUpdate)
                        .secure(true)
                        .width(400)
                        .padding([5, 10])
                ].spacing(5),
                container(
                    button(text!("Finish").size(20))
                        .on_press(Message::FinishServerCreation)
                )
                .width(Length::Fill)
                .align_x(Alignment::Center)
            ]
            .spacing(15)
            .padding(padding::all(50).top(0))
        ])
        .width(1000)
        .padding(padding::all(10))
        .height(Length::Fill)
        .style(|_theme| tf2::container::main(_theme))
    )
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(40)
    .style(|theme| tf2::container::surface(theme))
    .into()
}

pub fn download_server(path: PathBuf, appid: Game) -> impl Straw<(), f32, Error> {
    let testun = path
        .to_str()
        .and_then(|string| Some(string.to_string()))
        .unwrap_or("server".to_string());

    let appid = appid.clone();

    sipper(async move |mut progress| {
        #[cfg(target_os = "windows")]
        {
            // It was quicker to implement it here. I should move this in its own thingy down the line.

            const SRCDS_FIX_LINK: &str = "https://github.com/tsuza/srcds-pipe-passthrough-fix/releases/latest/download/srcds-fix-x86.exe";

            let srcds_fix_contents = reqwest::get(SRCDS_FIX_LINK)
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap();

            let _ = std::fs::write(format!("{}/srcds-fix.exe", testun), srcds_fix_contents);
        }

        let mut depot = DepotDownloader::new("./depotdownloader")
            .await
            .context(ServerDownloadSnafu)?;

        let stdout = depot
            .download_app(&testun, appid.into())
            .await
            .context(ServerDownloadSnafu)?;

        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();

            while let Some(line) = reader.next_line().await.context(IoSnafu)? {
                if let Some(percent) = line.split("%").next() {
                    if let Ok(percent) = percent.trim().parse::<f32>() {
                        let _ = progress.send(percent).await;
                    }
                }
            }
        }

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum Update {
    Downloading(f32),
    Finished(Result<(), Error>),
}

impl Default for State {
    fn default() -> Self {
        Self {
            form_page: FormSection::GameSelection,
            server: ServerInfo::default(),
            is_downloading: false,
            progress: 0.0,
        }
    }
}

#[derive(Snafu, Debug, Clone)]
pub enum Error {
    #[snafu(display("There was an error while creating the server: {source}"))]
    ServerDownloadError { source: core::Error },

    #[snafu(display("io error: {source}"))]
    Io {
        #[snafu(source(from(io::Error, Arc::new)))]
        source: Arc<io::Error>,
    },
}
