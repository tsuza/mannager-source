use ui::{style::icon, State};

pub mod core;
pub mod ui;

const TF2_BUILD_FONT_BYTES: &[u8] = include_bytes!("../fonts/tf2build.ttf");

fn main() -> iced::Result {
    iced::daemon(State::title, State::update, State::view)
        .subscription(State::subscription)
        .font(icon::FONT_BYTES)
        .font(TF2_BUILD_FONT_BYTES)
        .run_with(State::new)
}
