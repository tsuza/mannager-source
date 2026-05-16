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
    let primary = theme.colors().primary;
    let secondary = theme.colors().secondary;
    let outline = theme.colors().outline;

    Style {
        moved_item_overlay: primary.container.scale_alpha(0.18),

        ghost_border: Border {
            width: 2.0,
            color: outline.color,
            radius: 10.0.into(),
        },

        ghost_background: secondary.container.scale_alpha(0.18).into(),

        scale: 1.0,
    }
}
