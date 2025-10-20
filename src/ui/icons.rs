use iced::Font;
use iced::widget::text;

use crate::ui::themes::Theme;

type Text<'a> = iced::widget::Text<'a, Theme>;

pub const FONT_BYTES: &[u8] = include_bytes!("../../fonts/mannagericons.ttf");

pub fn bug<'a>() -> Text<'a> {
    with_codepoint('\u{E800}')
}

pub fn left_arrow<'a>() -> Text<'a> {
    with_codepoint('\u{E801}')
}

pub fn right_arrow<'a>() -> Text<'a> {
    with_codepoint('\u{E802}')
}

pub fn warning<'a>() -> Text<'a> {
    with_codepoint('\u{E803}')
}

pub fn copy<'a>() -> Text<'a> {
    with_codepoint('\u{E804}')
}

pub fn download<'a>() -> Text<'a> {
    with_codepoint('\u{E805}')
}

pub fn folder<'a>() -> Text<'a> {
    with_codepoint('\u{E806}')
}

pub fn password<'a>() -> Text<'a> {
    with_codepoint('\u{E807}')
}

pub fn link<'a>() -> Text<'a> {
    with_codepoint('\u{E808}')
}

pub fn location<'a>() -> Text<'a> {
    with_codepoint('\u{E809}')
}

pub fn menu<'a>() -> Text<'a> {
    with_codepoint('\u{E80A}')
}

pub fn minus<'a>() -> Text<'a> {
    with_codepoint('\u{E80B}')
}

pub fn edit<'a>() -> Text<'a> {
    with_codepoint('\u{E80C}')
}

pub fn start<'a>() -> Text<'a> {
    with_codepoint('\u{E80D}')
}

pub fn plus<'a>() -> Text<'a> {
    with_codepoint('\u{E80E}')
}

pub fn save<'a>() -> Text<'a> {
    with_codepoint('\u{E80F}')
}

pub fn stop<'a>() -> Text<'a> {
    with_codepoint('\u{E810}')
}

pub fn terminal<'a>() -> Text<'a> {
    with_codepoint('\u{E811}')
}

pub fn trash<'a>() -> Text<'a> {
    with_codepoint('\u{E812}')
}

pub fn close<'a>() -> Text<'a> {
    with_codepoint('\u{E813}')
}

pub fn people<'a>() -> Text<'a> {
    with_codepoint('\u{E814}')
}

pub fn port<'a>() -> Text<'a> {
    with_codepoint('\u{E815}')
}

pub fn book<'a>() -> Text<'a> {
    with_codepoint('\u{E816}')
}

pub fn check<'a>() -> Text<'a> {
    with_codepoint('\u{E817}')
}

fn with_codepoint<'a>(codepoint: char) -> Text<'a> {
    const FONT: Font = Font::with_name("mannagericons");

    text(codepoint).font(FONT)
}
