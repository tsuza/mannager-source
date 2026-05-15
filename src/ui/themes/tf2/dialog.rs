use iced::{color, widget::container};
use iced_dialog::dialog::{Catalog, Style, StyleFn};

use crate::ui::themes::Theme;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> <Self as Catalog>::Class<'a> {
        Box::new(default)
    }

    fn default_container<'a>() -> <Self as container::Catalog>::Class<'a> {
        Box::new(|theme| container::background(theme.colors().surface.surface_container.base))
    }

    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style {
        class(self)
    }
}

/// The default style of a [`Dialog`].
pub fn default<Theme>(_theme: &Theme) -> Style {
    Style {
        backdrop_color: color!(0x000000, 0.3),
    }
}
