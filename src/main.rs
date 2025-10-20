#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use iced::{Font, Size};
use ui::State;

use crate::ui::icons::FONT_BYTES;

pub mod core;
pub mod ui;

pub const APPLICATION_ID: &str = "org.tsuza.mannager";
pub const APP_ICON_BYTES: &[u8] = include_bytes!("../assets/app_icon.png");

const TF2_BUILD_FONT_BYTES: &[u8] = include_bytes!("../fonts/tf2build.ttf");
const TF2_SECONDARY_FONT_BYTES: &[u8] = include_bytes!("../fonts/TF2secondary.ttf");

fn main() -> iced::Result {
    let window_settings = iced::window::Settings {
        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: APPLICATION_ID.to_string(),
            override_redirect: false,
        },
        #[cfg(target_os = "windows")]
        icon: window::icon::from_file_data(APP_ICON_BYTES, Some(ImageFormat::Png)).ok(),
        ..Default::default()
    };

    iced::application(State::new, State::update, State::view)
        .title(State::title)
        .subscription(State::subscription)
        .window_size(Size::new(900.0, 900.0))
        .centered()
        .font(FONT_BYTES)
        .font(TF2_BUILD_FONT_BYTES)
        .font(TF2_SECONDARY_FONT_BYTES)
        .font(iced_aw::ICED_AW_FONT_BYTES)
        .default_font(Font::with_name("TF2 Secondary"))
        .window(window_settings)
        .run()
}
