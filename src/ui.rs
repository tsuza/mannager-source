use iced::{widget, Element, Subscription, Task};
use screen::{serverlist, Screen, ScreenKind};

pub mod components;
pub mod screen;

pub struct State {
    screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Message {
    ServerList(serverlist::Message),
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let (screen, _) = screen::serverlist::ServerList::new();
        (
            Self {
                screen: Screen {
                    current_page: screen::ScreenKind::ServerList,
                    serverlist_page: screen,
                },
            },
            widget::focus_next(),
        )
    }

    pub fn title(&self) -> String {
        "MANNager".into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerList(msg) => self
                .screen
                .serverlist_page
                .update(msg)
                .map(Message::ServerList),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn view(&self) -> Element<Message> {
        match &self.screen.current_page {
            ScreenKind::ServerList => self.screen.serverlist_page.view().map(Message::ServerList),
        }
    }
}
