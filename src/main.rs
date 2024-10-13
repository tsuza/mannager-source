use iced::Font;
use ui::{style::icon, State};

pub mod core;
pub mod ui;

const TF2_BUILD_FONT_BYTES: &[u8] = include_bytes!("../fonts/tf2build.ttf");
const TF2_SECONDARY_FONT_BYTES: &[u8] = include_bytes!("../fonts/TF2Secondary.ttf");
const IOSEVKA_MONO_FONT_BYTES: &[u8] = include_bytes!("../fonts/Iosevka-Regular.ttc");

fn main() -> iced::Result {
    iced::daemon(State::title, State::update, State::view)
        .subscription(State::subscription)
        .theme(State::theme)
        .default_font(Font::with_name("TF2 Secondary"))
        .font(icon::FONT_BYTES)
        .font(TF2_BUILD_FONT_BYTES)
        .font(TF2_SECONDARY_FONT_BYTES)
        .font(IOSEVKA_MONO_FONT_BYTES)
        .run_with(State::new)
}
