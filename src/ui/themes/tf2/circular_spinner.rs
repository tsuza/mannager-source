use crate::ui::components::spinner::{Appearance, StyleSheet};

use super::super::Theme;

impl StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> Appearance {
        let colors = self.colors();

        Appearance {
            background: None,
            track_color: colors.surface.on_surface_variant,
            bar_color: colors.primary.color,
        }
    }
}
