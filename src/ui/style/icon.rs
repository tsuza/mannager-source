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

pub fn warning<'a>() -> Text<'a> {
    with_codepoint('\u{E809}')
}

fn with_codepoint<'a>(codepoint: char) -> Text<'a> {
    const FONT: Font = Font::with_name("mannagericons");

    text(codepoint).font(FONT)
}
