use std::{fs, path::PathBuf};

use iced::{
    border, color, padding,
    widget::{
        button, center, column, container, horizontal_rule,
        rule::{self, FillMode},
        scrollable, text, vertical_space,
    },
    Alignment, Element, Length, Shadow, Subscription, Task, Vector,
};
use serde::{Deserialize, Serialize};

use crate::ui::components::modal::modal;

use super::servercreation::{self, FormPage};

pub struct ServerList {
    is_server_creation_popup_visible: bool,
    server_creation_screen: servercreation::State,
    servers: Vec<ServerInfo>,
}

#[derive(Deserialize, Serialize, Clone)]
struct ServerInfo {
    pub name: String,
    pub game: SourceAppIDs,
    pub path: PathBuf,
    pub map: String,
    pub max_players: u32,
    pub password: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateServer,
    OnClickOutsidePopup,
    ServerCreation(servercreation::Message),
    TestingGrounds(()),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub enum SourceAppIDs {
    #[default]
    TeamFortress2,
    CounterStrike2,
    LeftForDead1,
    LeftForDead2,
}

impl From<SourceAppIDs> for u32 {
    fn from(value: SourceAppIDs) -> Self {
        match value {
            SourceAppIDs::TeamFortress2 => 232250,
            SourceAppIDs::CounterStrike2 => 730,
            SourceAppIDs::LeftForDead1 => 222840,
            SourceAppIDs::LeftForDead2 => 222860,
        }
    }
}

impl ServerList {
    pub fn new() -> (Self, Task<Message>) {
        #[derive(Deserialize)]
        struct DummyDeserializeStruct {
            servers: Vec<ServerInfo>,
        }

        let mut servers: Vec<ServerInfo> = vec![];

        if let Ok(servers_from_config) = fs::read_to_string(PathBuf::from(
            "/home/suza/Coding/Rust/mannager-source/Servers/server_list.toml",
        )) {
            if let Ok(str_config) = toml::from_str::<DummyDeserializeStruct>(&servers_from_config) {
                servers = str_config.servers
            }
        }

        (
            Self {
                is_server_creation_popup_visible: false,
                server_creation_screen: servercreation::State::new(),
                servers: servers,
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
                self.is_server_creation_popup_visible = !self.is_server_creation_popup_visible;

                Task::none()
            }
            Message::OnClickOutsidePopup => {
                if !self.server_creation_screen.form_info.is_downloading {
                    self.is_server_creation_popup_visible = false;
                    self.server_creation_screen.form_page = FormPage::GameSelection;
                }

                Task::none()
            }
            Message::ServerCreation(server_creation_message) => match server_creation_message {
                servercreation::Message::FinishServerCreation => {
                    self.is_server_creation_popup_visible = false;

                    self.servers.push(ServerInfo {
                        name: self.server_creation_screen.form_info.server_name.clone(),
                        game: self.server_creation_screen.form_info.source_game.clone(),
                        path: self.server_creation_screen.form_info.server_path.clone(),
                        map: self.server_creation_screen.form_info.map_name.clone(),
                        max_players: self.server_creation_screen.form_info.max_players.clone(),
                        password: self.server_creation_screen.form_info.password.clone(),
                    });

                    #[derive(Deserialize, Serialize)]
                    struct DummyDeserializeStruct {
                        servers: Vec<ServerInfo>,
                    }

                    let fartimus = DummyDeserializeStruct {
                        servers: Vec::from_iter(self.servers.iter().map(|item| item.clone())),
                    };

                    Task::perform(
                        async move {
                            tokio::fs::write(
                                "/home/suza/Coding/Rust/mannager-source/Servers/server_list.toml",
                                toml::to_string_pretty(&fartimus).unwrap(),
                            )
                            .await
                            .unwrap();
                        },
                        Message::TestingGrounds,
                    )
                }
                _ => self
                    .server_creation_screen
                    .update(server_creation_message)
                    .map(Message::ServerCreation),
            },
            Message::TestingGrounds(_) => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.server_creation_screen
            .subscription()
            .map(Message::ServerCreation)
    }

    pub fn view(&self) -> Element<Message> {
        let base = container(column![
            navbar(),
            container(
                container(scrollable(show_servers(&self.servers)))
                    .width(900)
                    .padding(50)
                    .style(|_theme| {
                        container::background(color!(0x2A2725))
                            .border(border::width(3).rounded(3).color(color!(0x363230)))
                            .shadow(Shadow {
                                color: color!(0x0),
                                offset: Vector::new(0.0, 3.0),
                                blur_radius: 5.0,
                            })
                    })
            )
            .align_x(Alignment::Center)
            .padding(40)
            .width(Length::Fill)
            .height(Length::Fill)
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::background(color!(0x1c1a19)));

        if self.is_server_creation_popup_visible {
            modal(
                base,
                self.server_creation_screen
                    .view()
                    .map(Message::ServerCreation),
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
            color: color!(0x363230),
            width: 3,
            fill_mode: FillMode::Full,
            ..rule::default(_theme)
        })
    ])
    .width(Length::Fill)
    .height(64)
    .padding(0)
    .style(|_theme| {
        container::background(color!(0x2A2725)).shadow(Shadow {
            color: color!(0x0),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 5.0,
        })
    })
    .into()
}

fn show_servers<'a>(servers: &Vec<ServerInfo>) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    column(servers.iter().map(|server| {
        container(column![
            text!("{}", server.name.clone()),
            text!("{}", server.max_players)
        ])
        .width(Length::Fill)
        .max_width(720)
        .height(128)
        .padding(10)
        .style(|_theme| {
            container::background(color!(0x2A2725))
                .border(border::width(1).rounded(10).color(color!(0x363230)))
                .shadow(Shadow {
                    color: color!(0, 0, 0, 0.5),
                    offset: Vector::new(0.0, 3.0),
                    blur_radius: 5.0,
                })
        })
        .into()
    }))
    .push(
        button("+")
            .on_press(Message::CreateServer)
            .padding([15, 80]),
    )
    .align_x(Alignment::Center)
    .spacing(10)
    .into()
}
