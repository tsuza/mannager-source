use iced::widget::{text, Text};
use iced::Font;

pub const FONT_BYTES: &[u8] = include_bytes!("../../../fonts/mannagericons.ttf");

pub fn settings<'a>() -> Text<'a> {
    with_codepoint('\u{E800}')
}

pub fn start<'a>() -> Text<'a> {
    with_codepoint('\u{E801}')
}

pub fn modify<'a>() -> Text<'a> {
    with_codepoint('\u{E802}')
}

pub fn save<'a>() -> Text<'a> {
    with_codepoint('\u{E803}')
}

pub fn download<'a>() -> Text<'a> {
    with_codepoint('\u{E804}')
}

pub fn loading<'a>() -> Text<'a> {
    with_codepoint('\u{E805}')
}

pub fn stop<'a>() -> Text<'a> {
    with_codepoint('\u{E806}')
}

pub fn right_arrow<'a>() -> Text<'a> {
    with_codepoint('\u{E807}')
}

pub fn window_close<'a>() -> Text<'a> {
    with_codepoint('\u{E808}')
}

pub fn window_maximize<'a>() -> Text<'a> {
    with_codepoint('\u{F2D0}')
}

pub fn window_minimize<'a>() -> Text<'a> {
    with_codepoint('\u{F2D1}')
}

fn with_codepoint<'a>(codepoint: char) -> Text<'a> {
    const FONT: Font = Font::with_name("mannagericons");

    text(codepoint).font(FONT)
}
