use ui::State;

pub mod core;
pub mod ui;

fn main() -> iced::Result {
    iced::application(State::title, State::update, State::view).run_with(State::new)
}
