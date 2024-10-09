use iced::{window, Element, Subscription, Task};
use screen::{serverlist, Screen, ScreenKind};

pub mod components;
pub mod screen;
pub mod style;

pub struct State {
    screen: (Option<window::Id>, Screen),
}

#[derive(Debug, Clone)]
pub enum Message {
    ServerList(window::Id, serverlist::Message),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let (id, open) = window::open(window::Settings::default());

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
            Message::ServerList(window_id, msg) => {
                let serverlist_page = &mut self.screen.1.serverlist_page;

                serverlist_page
                    .update(msg)
                    .map(move |_mgs: serverlist::Message| Message::ServerList(window_id, _mgs))
            }
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

                let are_terminals_open = serverlist_page
                    .servers
                    .iter()
                    .any(|server| server.terminal_window.0 != None);

                if are_terminals_open || self.screen.0 != None {
                    _task
                } else {
                    _task.chain(iced::exit())
                }
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
