#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use iced::{Font, Size};
#[cfg(target_os = "windows")]
use iced::{advanced::graphics::image::image_rs::ImageFormat, window};

use ui::State;
use velopack::VelopackApp;

pub mod core;
pub mod icon;
pub mod ui;
pub mod update;
pub mod utils;

pub const APPLICATION_ID: &str = "net.tsuza.mannager";
pub const APP_ICON_BYTES: &[u8] = include_bytes!("../assets/app_icon.png");

pub const UPDATES_URL: &str = "https://github.com/tsuza/mannager-source/releases/latest/download";

const TF2_BUILD_FONT_BYTES: &[u8] = include_bytes!("../fonts/tf2build.ttf");
const TF2_SECONDARY_FONT_BYTES: &[u8] = include_bytes!("../fonts/TF2secondary.ttf");
const ROBOMONO_FONT_BYTES: &[u8] = include_bytes!("../fonts/robomono.ttf");

fn main() -> iced::Result {
    VelopackApp::build().run();

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
        .window(window_settings)
        .window_size(Size::new(900.0, 900.0))
        .centered()
        .default_font(Font::new("TF2 Secondary"))
        .font(icon::FONT)
        .font(TF2_BUILD_FONT_BYTES)
        .font(TF2_SECONDARY_FONT_BYTES)
        .font(ROBOMONO_FONT_BYTES)
        .font(iced_aw::ICED_AW_FONT_BYTES)
        .run()
}
