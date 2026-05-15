use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::icon;
use crate::ui::Element;
use crate::ui::components::notification::notification;
use crate::ui::components::progress_bar::animated_progress_bar;
use crate::ui::components::progress_stepper::stepper;
use crate::ui::games::{SOURCE_GAMES, SourceGame};
use crate::ui::server::ServerInfo;
use crate::ui::themes::{Theme, tf2};
use iced::widget::text::Wrapping;
use iced::widget::{Row, float, rule, space, tooltip};
use iced::{
    Alignment, ContentFit, Length, Task, padding,
    task::{Straw, sipper},
    widget::{button, center, column, container, row, svg, text, text_input},
};
use iced::{Font, Shadow};
use iced_aw::number_input;
use rfd::FileHandle;
use snafu::{ResultExt, Snafu};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::core::depotdownloader::DepotDownloader;
use crate::core::{self, Game};

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

#[derive(Default, PartialEq, Eq, Ord, PartialOrd)]
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
            Message::SelectMap => {
                let path = self
                    .server
                    .path
                    .join(&self.server.game.arg_name())
                    .join("maps");

                Action::Run(Task::perform(
                    rfd::AsyncFileDialog::new()
                        .set_title("Choose a default map")
                        .set_directory(path)
                        .add_filter("Source Map", &["bsp", "vpk"])
                        .pick_file(),
                    Message::SelectMapFinished,
                ))
            }
            Message::SelectMapFinished(file_handle) => {
                let Some(file) = file_handle else {
                    return Action::None;
                };

                let Some(map) = file
                    .path()
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_owned)
                else {
                    return Action::None;
                };

                self.server.map = map;

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
                    .then_some(port)
                    .and_then(|port| port.parse::<u16>().ok());

                Action::None
            }
            Message::GsltUpdate(token) => {
                self.server.gslt = (!token.is_empty()).then_some(token);

                Action::None
            }
            Message::CloseServerCreation => Action::SwitchToServerList,
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        match self.form_page {
            FormSection::GameSelection => choose_game_view(&self.server),
            FormSection::Downloading => downloading_view(self.progress),
            FormSection::ServerInfo => info_view(&self.server),
        }
    }
}

fn choose_game_view<'a>(server: &ServerInfo) -> Element<'a, Message> {
    fn game_entry<'a>(
        game: &SourceGame,
        is_currently_selected: bool,
        button_event: Message,
    ) -> Element<'a, Message> {
        if is_currently_selected {
            button(
                float(
                    svg(game.image.clone())
                        .content_fit(ContentFit::Contain)
                        .height(80)
                        .width(80),
                )
                .scale(1.1),
            )
            .on_press(button_event)
            .padding(padding::vertical(14).horizontal(12))
            .style(|theme, _| tf2::button::default(theme, button::Status::Pressed))
            .into()
        } else {
            tooltip(
                button(
                    svg(game.image.clone())
                        .content_fit(ContentFit::Contain)
                        .height(80)
                        .width(80)
                        .opacity(0.5),
                )
                .on_press(button_event)
                .padding(padding::vertical(14).horizontal(12))
                .style(|theme, _| tf2::button::default(theme, button::Status::Active)),
                container(text(game.game.to_string())).padding(10),
                tooltip::Position::Bottom,
            )
            .gap(10)
            .padding(10)
            .style(|theme| tf2::container::tooltip(theme))
            .into()
        }
    }

    let games = Row::with_children(SOURCE_GAMES.iter().map(|game| {
        game_entry(
            game,
            server.game == game.game,
            Message::GameChosen(game.game),
        )
    }))
    .spacing(20)
    .align_y(Alignment::Center)
    .wrap()
    .align_x(Alignment::Center);

    let header = {
        let title = container(column![
            text("Create server")
                .font(Font::new("TF2 Build"))
                .line_height(1.0)
                .size(30),
            text("Configure and launch a new instance")
                .size(12)
                .style(tf2::text::muted)
        ]);

        let close_button = button(
            icon::left_arrow()
                .width(Length::Fill)
                .height(Length::Fill)
                .size(20)
                .center(),
        )
        .on_press(Message::CloseServerCreation)
        .width(34)
        .height(34);

        column![
            row![close_button, title]
                .align_y(Alignment::Center)
                .spacing(14)
                .width(Length::Fill),
            rule::horizontal(1),
        ]
        .width(Length::Fill)
        .spacing(5)
    };

    let creation_progress_bar = {
        stepper(
            [
                ("Configure", FormSection::GameSelection),
                ("Download", FormSection::Downloading),
                ("Options", FormSection::ServerInfo),
            ],
            FormSection::GameSelection,
        )
    };

    let body = {
        let name_input = column![
            text!("Server Name").style(tf2::text::secondary),
            text_input("e.g. My Server", &server.name)
                .on_input(Message::ServerNameInput)
                .width(Length::Fill)
                .padding(padding::vertical(10).horizontal(13))
        ]
        .spacing(5);

        let path_picker = {
            let path = server.path.display().to_string();

            let path = container(text(path))
                .width(Length::Fill)
                .padding(padding::vertical(10).horizontal(13))
                .style(tf2::container::main);

            column![
                text("Server Path").style(tf2::text::secondary),
                row![
                    button(
                        row![icon::folder(), "Browse..."]
                            .align_y(Alignment::Center)
                            .spacing(7)
                    )
                    .on_press(Message::ChooseServerPath)
                    .padding(padding::vertical(12).horizontal(14)),
                    path
                ]
                .align_y(Alignment::Center)
                .spacing(10)
            ]
            .spacing(5)
        };

        let game_section = container(column![
            row![
                text("Game").style(tf2::text::secondary),
                tooltip(
                    icon::warning().style(tf2::text::secondary),
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
                games
            )
            .center_x(Length::Fill)
        ].spacing(10));

        container(column![name_input, path_picker, rule::horizontal(1), game_section].spacing(30))
    };

    container(
        column![
            header,
            creation_progress_bar,
            container(
                column![
                    body,
                    rule::horizontal(1),
                    container(
                        button(
                            row![text("Next").size(20), icon::right_arrow().size(20)].spacing(10)
                        )
                        .on_press(Message::DownloadServer)
                        .padding(padding::vertical(10).horizontal(20))
                        .style(tf2::button::primary)
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::End)
                ]
                .spacing(20)
            )
            .padding(24)
            .style(tf2::container::card),
        ]
        .width(620)
        .spacing(14),
    )
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(40)
    .style(tf2::container::main)
    .into()
}

// TODO: Finish styling this part
fn downloading_view<'a>(progress: f32) -> Element<'a, Message> {
    let header = container(
        row![
            text("Create Server")
                .font(Font::new("TF2 Build"))
                .wrapping(Wrapping::None)
                .line_height(1.0)
                .size(30)
                .width(Length::Fill)
                .align_y(Alignment::Center),
            space::horizontal(),
            container(stepper(
                [
                    ("Configure", FormSection::GameSelection),
                    ("Download", FormSection::Downloading),
                    ("Options", FormSection::ServerInfo),
                ],
                FormSection::Downloading,
            ))
            .width(400)
        ]
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(padding::vertical(12).horizontal(16))
    .style(|theme: &Theme| {
        let mut style = tf2::container::card(theme).shadow(Shadow::default());

        style.border = style.border.rounded(0);

        style
    });

    let progress = animated_progress_bar(0.0..=100.0, progress)
        .length(Length::Fill)
        .girth(50);

    container(
        column![
            header,
            center(
                container(progress)
                    .width(620)
                    .padding(24)
                    .style(tf2::container::card),
            )
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .style(tf2::container::main)
    .into()
}

fn info_view<'a>(server: &'a ServerInfo) -> Element<'a, Message> {
    let header = {
        let title = container(column![
            text("Create server")
                .font(Font::new("TF2 Build"))
                .line_height(1.0)
                .size(30),
            text!("{} · {}", server.game, server.path.display())
                .size(12)
                .style(tf2::text::muted)
        ])
        .width(Length::Fill);

        column![title, rule::horizontal(1),]
            .width(Length::Fill)
            .spacing(5)
    };

    let creation_progress_bar = {
        stepper(
            [
                ("Configure", FormSection::GameSelection),
                ("Download", FormSection::Downloading),
                ("Options", FormSection::ServerInfo),
            ],
            FormSection::ServerInfo,
        )
    };

    let body = {
        fn optional_tag<'a>() -> Element<'a, Message> {
            container(text("optional").size(10).style(tf2::text::muted))
                .padding(padding::vertical(1).horizontal(6))
                .style(tf2::container::main)
                .into()
        }

        let description_input = column![
            row![
                text("Server Description").style(tf2::text::secondary),
                optional_tag()
            ]
            .align_y(Alignment::Center)
            .spacing(5),
            text_input(
                "e.g. Testing Server",
                &server.description.as_deref().unwrap_or_default()
            )
            .on_input(Message::MessageDescriptionUpdate)
            .width(Length::Fill)
            .padding(padding::vertical(10).horizontal(13))
        ]
        .spacing(5);

        let map_input = column![
            text("Map").style(tf2::text::secondary),
            row![
                container(
                    button(
                        row![icon::map(), "Select Map"]
                            .align_y(Alignment::Center)
                            .spacing(7)
                    )
                    .on_press(Message::SelectMap)
                    .padding(padding::vertical(12).horizontal(14)),
                ),
                container(text(server.map.as_str()))
                    .width(Length::Fill)
                    .padding(padding::vertical(10).horizontal(13))
                    .style(tf2::container::main)
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        ]
        .spacing(5);

        let max_players_input = column![
            text("Max Players").style(tf2::text::secondary),
            container(
                number_input(&server.max_players, 0..=100, Message::MaxPlayersUpdate)
                    .padding(padding::vertical(10).horizontal(13))
            )
        ]
        .spacing(5);

        let password_input = column![
            row![
                text("Server Password").style(tf2::text::secondary),
                optional_tag()
            ]
            .align_y(Alignment::Center)
            .spacing(5),
            text_input(
                "e.g. password123",
                &server.password.as_deref().unwrap_or_default()
            )
            .on_input(Message::PasswordUpdate)
            .secure(true)
            .width(Length::Fill)
            .padding(padding::vertical(10).horizontal(13))
        ]
        .spacing(5);

        let port_input = column![
            row![
                text("Port").style(tf2::text::secondary),
                optional_tag(),
                tooltip(
                    icon::warning().style(tf2::text::secondary),
                    text("If it's left empty, the app will automatically find an available port.")
                        .width(350),
                    tooltip::Position::Top
                )
                .gap(10)
                .padding(20)
                .style(|_theme| tf2::container::tooltip(_theme))
            ]
            .spacing(5)
            .align_y(Alignment::Center),
            text_input(
                "27015",
                &server.port.map(|port| port.to_string()).unwrap_or_default()
            )
            .on_input(Message::PortUpdate)
            .width(150)
            .padding(padding::vertical(10).horizontal(13))
        ]
        .spacing(5);

        // TODO: put "Required to appear in the public server browser in some games. Get a token →"
        let gslt_input = column![
            row![text("GSLT").style(tf2::text::secondary), optional_tag()]
                .spacing(5)
                .align_y(Alignment::Center),
            text_input("GSLT", &server.gslt.as_deref().unwrap_or_default())
                .on_input(Message::GsltUpdate)
                .secure(true)
                .width(Length::Fill)
                .padding(padding::vertical(10).horizontal(13))
        ]
        .spacing(5);

        container(
            column![
                description_input,
                max_players_input,
                map_input,
                rule::horizontal(1),
                row![password_input, port_input]
                    .width(Length::Fill)
                    .spacing(20),
                gslt_input
            ]
            .spacing(30),
        )
    };

    container(
        column![
            header,
            creation_progress_bar,
            container(
                column![
                    body,
                    rule::horizontal(1),
                    container(
                        button(
                            row![text("Finish").size(20), icon::right_arrow().size(20)].spacing(10)
                        )
                        .on_press(Message::FinishServerCreation)
                        .padding(padding::vertical(10).horizontal(20))
                        .style(tf2::button::primary)
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::End)
                ]
                .spacing(20)
            )
            .padding(24)
            .style(tf2::container::card),
        ]
        .width(620)
        .spacing(14),
    )
    .align_x(Alignment::Center)
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(40)
    .style(tf2::container::main)
    .into()
}

pub fn download_server(path: PathBuf, appid: Game) -> impl Straw<(), f32, Error> {
    let testun = path.to_str().unwrap_or("server").to_string();

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
