use iced::{window, Element, Subscription, Task};
use screen::{serverlist, Screen, ScreenKind};

#[cfg(target_os = "windows")]
use iced::advanced::graphics::image::image_rs::ImageFormat;

#[cfg(target_os = "linux")]
use crate::APPLICATION_ID;

#[cfg(target_os = "windows")]
use crate::APP_ICON_BYTES;

pub mod components;
pub mod screen;
pub mod style;

pub struct State {
    screen: (Option<window::Id>, Screen),
}

#[derive(Debug, Clone)]
pub enum Message {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    ServerList(window::Id, serverlist::Message),
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let (id, open) = window::open(window::Settings {
            #[cfg(target_os = "linux")]
            platform_specific: window::settings::PlatformSpecific {
                application_id: APPLICATION_ID.to_string(),
                override_redirect: false,
            },
            #[cfg(target_os = "windows")]
            icon: window::icon::from_file_data(APP_ICON_BYTES, Some(ImageFormat::Png)).ok(),
            ..Default::default()
        });

        let (main_screen_state, main_screen_task) = serverlist::State::new();

        (
            Self {
                screen: (
                    Some(id),
                    Screen {
                        current_page: ScreenKind::ServerList,
                        serverlist_page: main_screen_state,
                    },
                ),
            },
            main_screen_task
                .map(move |x| Message::ServerList(id, x))
                .chain(open.map(Message::WindowOpened)),
        )
    }

    pub fn title(&self, _window: window::Id) -> String {
        "MANNager".into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(_id) => Task::none(),
            Message::WindowClosed(id) => {
                let mut _task = Task::none();

                let serverlist_page = &mut self.screen.1.serverlist_page;

                _task = if Some(id) == self.screen.0 {
                    self.screen.0 = None;

                    serverlist_page
                        .update(serverlist::Message::WindowClosed)
                        .map(move |x| Message::ServerList(id, x))
                } else {
                    serverlist_page
                        .update(serverlist::Message::TerminalClosed(id))
                        .map(move |x| Message::ServerList(id, x))
                };

                let is_a_terminal_open = serverlist_page
                    .servers
                    .iter()
                    .any(|server| server.is_running());

                if is_a_terminal_open || self.screen.0 != None {
                    _task
                } else {
                    _task.chain(iced::exit())
                }
            }
            Message::ServerList(window_id, msg) => {
                let serverlist_page = &mut self.screen.1.serverlist_page;

                serverlist_page
                    .update(msg)
                    .map(move |_mgs: serverlist::Message| Message::ServerList(window_id, _mgs))
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }

    pub fn view(&self, window_id: window::Id) -> Element<Message> {
        let serverlist_page = &self.screen.1.serverlist_page;

        serverlist_page
            .view(window_id)
            .map(move |msg| Message::ServerList(window_id, msg))
    }
}
