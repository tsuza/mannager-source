use iced::{
    border::Radius,
    widget::float::{Catalog, Style, StyleFn},
};

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

pub fn default(_theme: &Theme) -> Style {
    Style {
        shadow: iced::Shadow {
            blur_radius: 5.0,
            ..Default::default()
        },
        shadow_border_radius: Radius::new(5.0),
    }
}
