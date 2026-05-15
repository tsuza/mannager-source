use iced::{
    Background,
    widget::table::{Catalog, Style, StyleFn},
};

use crate::ui::themes::Theme;

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
    let separator = theme.colors().outline.variant;

    Style {
        separator_x: Background::Color(separator),
        separator_y: Background::Color(separator),
    }
}
