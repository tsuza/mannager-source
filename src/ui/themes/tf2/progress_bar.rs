use iced::widget::progress_bar::{Catalog, Style, StyleFn};
use iced::{Background, border};

use super::super::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    Style {
        background: Background::Color(theme.colors().secondary.secondary_container),
        bar: Background::Color(theme.colors().primary.color),
        border: border::rounded(0),
    }
}
