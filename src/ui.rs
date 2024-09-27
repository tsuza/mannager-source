use std::collections::BTreeMap;

use iced::{
    widget::{self, container},
    window, Element, Subscription, Task,
};
use screen::{serverboot, serverlist, Screen, ScreenKind};

pub mod components;
pub mod screen;
pub mod style;

pub struct State {
    windows: BTreeMap<window::Id, Window>,
}

enum Window {
    MainApp(Screen),
    ServerTerminal(serverboot::State),
}

#[derive(Debug, Clone)]
pub enum Message {
    ServerList(window::Id, serverlist::Message),
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    ServerTerminal(window::Id, serverboot::Message),
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let (_, open) = window::open(window::Settings::default());

        (
            Self {
                windows: BTreeMap::new(),
            },
            open.map(Message::WindowOpened),
        )
    }

    pub fn title(&self, window: window::Id) -> String {
        "MANNager".into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ServerList(window_id, msg) => {
                let Some(mainapp) = self.windows.get_mut(&window_id) else {
                    return Task::none();
                };

                let Window::MainApp(main_window) = mainapp else {
                    return Task::none();
                };

                let mut tasks: Vec<Task<Message>> = vec![];

                let msg_clone = msg.clone();

                tasks.push(
                    main_window
                        .serverlist_page
                        .update(msg_clone)
                        .map(move |msg6| Message::ServerList(window_id, msg6)),
                );

                if let serverlist::Message::ServerConsoleOpened(server_id, window_id2) = msg {
                    let (test, test1) = serverboot::State::new(
                        &main_window.serverlist_page.servers[server_id].info,
                    );

                    tasks.push(test1.map(move |msg5| Message::ServerTerminal(window_id2, msg5)));

                    self.windows
                        .insert(window_id2, Window::ServerTerminal(test));
                }

                Task::batch(tasks)
            }
            Message::WindowOpened(id) => {
                let (state, task) = serverlist::State::new();
                let screen = Screen {
                    current_page: ScreenKind::ServerList,
                    serverlist_page: state,
                };
                self.windows.insert(id, Window::MainApp(screen));

                task.map(move |msg| Message::ServerList(id, msg))
            }
            Message::WindowClosed(id) => {
                let window = self.windows.remove(&id);

                let mut task: Task<Message> = Task::none();

                if let Some(window1) = window {
                    match window1 {
                        Window::MainApp(mut screen) => {
                            task = screen
                                .serverlist_page
                                .update(serverlist::Message::WindowClosed)
                                .map(move |msg6| Message::ServerList(id, msg6));
                        }
                        Window::ServerTerminal(mut state) => {
                            task = state
                                .update(serverboot::Message::ShutDownServer)
                                .map(move |msg6| Message::ServerTerminal(id, msg6))
                        }
                    }
                }

                if self.windows.is_empty() {
                    Task::batch(vec![iced::exit(), task])
                } else {
                    Task::batch(vec![Task::none(), task])
                }
            }
            Message::ServerTerminal(window_id, msg) => {
                let Some(terminal_window) = self.windows.get_mut(&window_id) else {
                    return Task::none();
                };

                match terminal_window {
                    Window::MainApp(_) => Task::none(),
                    Window::ServerTerminal(state) => state
                        .update(msg)
                        .map(move |msg| Message::ServerTerminal(window_id, msg)),
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }

    pub fn view(&self, window_id: window::Id) -> Element<Message> {
        let Some(window) = self.windows.get(&window_id) else {
            return container("").into();
        };

        match window {
            Window::MainApp(screen) => screen
                .serverlist_page
                .view()
                .map(move |msg| Message::ServerList(window_id, msg)),
            Window::ServerTerminal(state) => state
                .view()
                .map(move |msg| Message::ServerTerminal(window_id, msg)),
        }
    }
}
