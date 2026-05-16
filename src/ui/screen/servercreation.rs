use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::icon;
use crate::ui::Element;
use crate::ui::components::notification::notification;
use crate::ui::components::progress_bar::animated_progress_bar;
use crate::ui::components::progress_stepper::stepper;
use crate::ui::components::spinner;
use crate::ui::games::{SOURCE_GAMES, SourceGame};
use crate::ui::server::ServerInfo;
use crate::ui::themes::{Theme, tf2};
use iced::widget::text::Wrapping;
use iced::widget::{Row, float, rule, scrollable, space, tooltip};
use iced::{
    Alignment, ContentFit, Length, Task, padding,
    task::{Straw, sipper},
    widget::{button, center, column, container, row, svg, text, text_input},
};
use iced::{Font, Shadow, border};
use iced_aw::number_input;
use rfd::FileHandle;
use snafu::{ResultExt, Snafu};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::core::depotdownloader::DepotDownloader;
use crate::core::{self, Game};

#[derive(Clone, Debug)]
pub struct DepotStatus {
    pub id: u32,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub struct DownloadUpdate {
    pub depots: Vec<DepotStatus>,
    pub phase: DownloadPhase,
    pub raw_line: String,
}

pub struct State {
    form_page: FormSection,
    server: ServerInfo,
    is_downloading: bool,
    download_depot_status: Vec<DepotStatus>,
    download_phase: DownloadPhase,
    download_log: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum DownloadPhase {
    #[default]
    Connecting,
    ResolvingDepots,
    Downloading,
    Validating,
    Done,
}

impl DownloadPhase {
    fn advance(self, trimmed: &str) -> Self {
        let next = if trimmed.starts_with("Connecting to Steam")
            || trimmed.starts_with("Logging")
            || trimmed.starts_with("Got AppInfo")
            || trimmed.starts_with("Got depot key")
        {
            DownloadPhase::Connecting
        } else if trimmed.starts_with("Processing depot")
            || trimmed.starts_with("Downloading depot")
            || trimmed.starts_with("Got manifest")
            || trimmed.starts_with("Manifest ")
            || trimmed.starts_with("Pre-allocating")
        {
            DownloadPhase::ResolvingDepots
        } else if trimmed.find('%').is_some()
            && trimmed
                .find('%')
                .and_then(|i| trimmed[..i].trim().parse::<f32>().ok())
                .is_some()
        {
            DownloadPhase::Downloading
        } else if trimmed.starts_with("Depot ") && trimmed.contains("Downloaded") {
            DownloadPhase::Downloading
        } else if trimmed.starts_with("Total downloaded") {
            DownloadPhase::Validating
        } else if trimmed == "Disconnected from Steam" {
            DownloadPhase::Done
        } else {
            return self;
        };

        next.max(self)
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            form_page: FormSection::GameSelection,
            server: ServerInfo::default(),
            is_downloading: false,
            download_depot_status: vec![],
            download_log: Vec::new(),
            download_phase: DownloadPhase::Connecting,
        }
    }
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
                Update::Downloading(status) => {
                    self.download_depot_status = status.depots;

                    self.download_log.push(status.raw_line);

                    self.download_phase = status.phase;

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
            FormSection::Downloading => downloading_view(
                &self.download_depot_status,
                &self.download_phase,
                &self.download_log,
                &self.server.game,
            ),
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
    .style(|theme| tf2::container::main(theme).border(border::width(0)))
    .into()
}

fn downloading_view<'a>(
    depot_status: &'a [DepotStatus],
    phase: &'a DownloadPhase,
    log: &'a [String],
    game: &'a Game,
) -> Element<'a, Message> {
    let header = container(
        row![
            text("Create Server")
                .font(Font::new("TF2 Build"))
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .wrapping(Wrapping::None)
                .line_height(1.0)
                .size(30),
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

    let game_logo = {
        let svg_handle = SOURCE_GAMES
            .iter()
            .find(|g| g.game == *game)
            .map(|g| g.image.clone());

        let logo_inner: Element<'a, Message> = if let Some(handle) = svg_handle {
            svg(handle)
                .content_fit(ContentFit::Contain)
                .width(50)
                .height(50)
                .into()
        } else {
            space::horizontal().width(50).height(50).into()
        };

        container(logo_inner).center(72)
    };

    let progress_section = {
        let status_label = {
            let label = match phase {
                DownloadPhase::Connecting => "Connecting to Steam…",
                DownloadPhase::ResolvingDepots => "Resolving depot manifests…",
                DownloadPhase::Downloading => "Downloading server files…",
                DownloadPhase::Validating => "Validating files…",
                DownloadPhase::Done => "Download complete!",
            };

            row![
                if *phase != DownloadPhase::Done {
                    Element::from(spinner::Circular::new().size(14.0))
                } else {
                    icon::check().size(14).style(tf2::text::success).into()
                },
                text(label).size(14).style(tf2::text::secondary),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        };

        let progress_bars: Element<'a, Message> = if depot_status.is_empty() {
            container(
                animated_progress_bar(0.0..=100.0, 0.0)
                    .length(Length::Fill)
                    .girth(8),
            )
            .into()
        } else {
            let depots = depot_status.iter().map(|depot| {
                row![
                    text(depot.id).width(90).size(11).style(tf2::text::muted),
                    animated_progress_bar(0.0..=100.0, depot.progress)
                        .length(Length::Fill)
                        .girth(8),
                    text!("{:.0}%", depot.progress)
                        .size(11)
                        .font(Font::MONOSPACE)
                        .style(tf2::text::primary)
                        .width(36)
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into()
            });

            column(depots).spacing(8).width(Length::Fill).into()
        };

        column![status_label, progress_bars]
            .spacing(8)
            .width(Length::Fill)
    };

    let log_tail = {
        let lines: Vec<Element<Message>> = log
            .iter()
            .rev()
            .take(60)
            .rev()
            .map(|line| {
                let (label, style): (&str, fn(&Theme) -> iced::widget::text::Style) =
                    if line.contains('%') {
                        ("DL", tf2::text::primary)
                    } else if line.contains("Done") || line.contains("Downloaded") {
                        ("OK", tf2::text::success)
                    } else {
                        ("SYS", tf2::text::muted)
                    };

                row![
                    container(text(label).center().size(9).font(Font::MONOSPACE))
                        .width(40)
                        .align_x(Alignment::Center)
                        .padding(padding::vertical(1))
                        .style(tf2::container::main),
                    text(line.trim())
                        .size(11)
                        .font(Font::MONOSPACE)
                        .style(style)
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .into()
            })
            .collect();

        container(
            scrollable(column(lines).width(Length::Fill))
                .anchor_bottom()
                .spacing(3),
        )
        .width(Length::Fill)
        .height(160)
        .padding(padding::all(10))
        .style(tf2::container::main)
    };

    let steps = {
        fn step_row<'a>(label: &'a str, done: bool, active: bool) -> Element<'a, Message> {
            let icon: Element<Message> = if done {
                icon::check().size(12).style(tf2::text::success).into()
            } else if active {
                spinner::Circular::new().size(12.0).into()
            } else {
                icon::circle().size(12).style(tf2::text::muted).into()
            };

            let label_style = if done {
                tf2::text::secondary
            } else if active {
                tf2::text::primary
            } else {
                tf2::text::muted
            };

            row![
                container(icon).width(18).center(18),
                text(label).size(12).style(label_style),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        }

        let connecting_done = *phase > DownloadPhase::Connecting;
        let connecting_active = *phase == DownloadPhase::Connecting;

        let resolving_done = *phase > DownloadPhase::ResolvingDepots;
        let resolving_active = *phase == DownloadPhase::ResolvingDepots;

        let dl_done = *phase > DownloadPhase::Downloading;
        let dl_active = *phase == DownloadPhase::Downloading;

        let val_done = *phase >= DownloadPhase::Done;
        let val_active = *phase == DownloadPhase::Validating;

        column![
            step_row(
                "Authenticating with Steam",
                connecting_done,
                connecting_active
            ),
            step_row(
                "Resolving depot manifests",
                resolving_done,
                resolving_active
            ),
            step_row("Downloading server files", dl_done, dl_active),
            step_row("Validating & writing files", val_done, val_active),
        ]
        .spacing(6)
        .width(Length::Fill)
    };

    let card = container(
        column![
            container(
                column![
                    game_logo,
                    text(game.to_string())
                        .size(20)
                        .font(Font::new("TF2 Build"))
                        .line_height(1.0),
                ]
                .spacing(14)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .align_x(Alignment::Center),
            rule::horizontal(1),
            progress_section,
            log_tail,
            rule::horizontal(1),
            steps,
        ]
        .spacing(16),
    )
    .width(520)
    .padding(24)
    .style(tf2::container::card);

    container(
        column![header, center(card).padding(50)]
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .style(|theme| tf2::container::main(theme).border(border::width(0)))
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
    .style(|theme| tf2::container::main(theme).border(border::width(0)))
    .into()
}

pub fn download_server(path: PathBuf, appid: Game) -> impl Straw<(), DownloadUpdate, Error> {
    let install_path = path.to_str().unwrap_or("server").to_string();
    let appid = appid.clone();

    sipper(async move |mut progress| {
        #[cfg(target_os = "windows")]
        {
            const SRCDS_FIX_LINK: &str = "https://github.com/tsuza/srcds-pipe-passthrough-fix/releases/latest/download/srcds-fix-x86.exe";

            let srcds_fix_contents = reqwest::get(SRCDS_FIX_LINK)
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap();

            let _ = std::fs::write(
                format!("{}/srcds-fix.exe", install_path),
                srcds_fix_contents,
            );
        }

        // TODO: Port SteamKit to Rust and use that instead
        let mut depot_downloader = DepotDownloader::new("./depotdownloader")
            .await
            .context(ServerDownloadSnafu)?;

        let stdout = depot_downloader
            .download_app(&install_path, appid.into())
            .await
            .context(ServerDownloadSnafu)?;

        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();

            let mut depots: Vec<DepotStatus> = Vec::new();
            let mut current_depot: Option<u32> = None;
            let mut phase = DownloadPhase::Connecting;

            while let Some(line) = reader.next_line().await.context(IoSnafu)? {
                let trimmed = line.trim();

                phase = phase.advance(trimmed);

                if let Some(rest) = trimmed.strip_prefix("Processing depot ") {
                    if let Ok(id) = rest.trim().parse::<u32>() {
                        if !depots.iter().any(|depot| depot.id == id) {
                            depots.push(DepotStatus { id, progress: 0.0 });
                        }
                        current_depot = Some(id);
                        let _ = progress
                            .send(DownloadUpdate {
                                depots: depots.clone(),
                                phase: phase.clone(),
                                raw_line: line.clone(),
                            })
                            .await;
                    }
                    continue;
                }

                if trimmed.starts_with("Downloading depot ") {
                    if let Some(id) = trimmed
                        .split("depot ")
                        .nth(1)
                        .and_then(|s| s.split_whitespace().next())
                        .and_then(|s| s.parse::<u32>().ok())
                    {
                        current_depot = Some(id);
                        let _ = progress
                            .send(DownloadUpdate {
                                depots: depots.clone(),
                                phase: phase.clone(),
                                raw_line: line.clone(),
                            })
                            .await;
                    }
                    continue;
                }

                if let Some(rest) = trimmed.strip_prefix("Depot ") {
                    if let Some(dash) = rest.find(" - Downloaded ") {
                        if let Ok(id) = rest[..dash].trim().parse::<u32>() {
                            if let Some(depot) = depots.iter_mut().find(|depot| depot.id == id) {
                                depot.progress = 100.0;
                            }

                            let _ = progress
                                .send(DownloadUpdate {
                                    depots: depots.clone(),
                                    phase: phase.clone(),
                                    raw_line: line.clone(),
                                })
                                .await;
                        }
                    }
                    continue;
                }

                if let Some(pct_end) = trimmed.find('%') {
                    if let Ok(pct) = trimmed[..pct_end].trim().parse::<f32>() {
                        if let Some(id) = current_depot {
                            if let Some(d) = depots.iter_mut().find(|d| d.id == id) {
                                d.progress = pct;
                            }
                        }
                        let _ = progress
                            .send(DownloadUpdate {
                                depots: depots.clone(),
                                phase: phase.clone(),
                                raw_line: line.clone(),
                            })
                            .await;
                    }
                    continue;
                }

                if !trimmed.starts_with("Pre-allocating") {
                    let _ = progress
                        .send(DownloadUpdate {
                            depots: depots.clone(),
                            phase: phase.clone(),
                            raw_line: line.clone(),
                        })
                        .await;
                }
            }
        }

        Ok(())
    })
}

#[derive(Debug, Clone)]
pub enum Update {
    Downloading(DownloadUpdate),
    Finished(Result<(), Error>),
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
