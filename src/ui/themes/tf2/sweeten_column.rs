use iced::Border;
use sweeten::widget::column::{Catalog, Style, StyleFn};

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
        moved_item_overlay: theme.colors().primary.primary_container.scale_alpha(0.2),
        ghost_border: Border {
            width: 4.0,
            color: theme.colors().outline.color,
            radius: 0.0.into(),
        },
        ghost_background: theme.colors().secondary.color.scale_alpha(0.2).into(),
        scale: 1.0,
    }
}
