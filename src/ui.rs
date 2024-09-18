use iced::{widget, Element, Task};

#[derive(Default)]
pub struct State;

#[derive(Debug, Clone)]
pub enum Message {}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        (Self, widget::focus_next())
    }

    pub fn title(&self) -> String {
        "Mannager - Source Engine Server manager".into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
        todo!()
    }
}
