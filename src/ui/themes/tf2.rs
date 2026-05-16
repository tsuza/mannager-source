use iced::color;

use crate::ui::themes::ColorQuartet;

use super::{ColorScheme, Inverse, Outline, Surface, SurfaceContainer, from_argb};

pub mod button;
pub mod checkbox;
pub mod circular_spinner;
pub mod container;
pub mod dialog;
pub mod float;
pub mod markdown;
pub mod menu;
pub mod metered_progress_bar;
pub mod number_input;
pub mod progress_bar;
pub mod rule;
pub mod scrollable;
pub mod selectable_text;
pub mod svg;
pub mod sweeten_column;
pub mod table;
pub mod text;
pub mod text_input;
pub mod toggler;

pub const fn color_scheme() -> ColorScheme {
    ColorScheme {
        primary: ColorQuartet {
            color: color!(0xcf6229),
            text: color!(0xf5ede4),
            container: color!(0x3d2010),
            container_text: color!(0xe08830),
        },
        secondary: ColorQuartet {
            color: color!(0x96897a),
            text: color!(0x1c1a17),
            container: color!(0x272320),
            container_text: color!(0xc4b8a8),
        },
        tertiary: ColorQuartet {
            color: color!(0x8a7a6a),
            text: color!(0xede4d4),
            container: color!(0x2a2320),
            container_text: color!(0xc4b8a8),
        },
        success: ColorQuartet {
            color: color!(0x4caf50),
            text: color!(0x0f1a0f),
            container: color!(0x1a2e1a),
            container_text: color!(0x80d480),
        },
        error: ColorQuartet {
            color: color!(0xc0392b),
            text: color!(0x1a0a0a),
            container: color!(0x2a1212),
            container_text: color!(0xe07070),
        },
        surface: Surface {
            color: color!(0x1c1a17),
            text: color!(0xede4d4),
            text_variant: color!(0x96897a),
            container: SurfaceContainer {
                lowest: color!(0x1a1815),
                low: color!(0x1f1c19),
                base: color!(0x272320),
                high: color!(0x302b28),
                highest: color!(0x38322b),
            },
        },
        inverse: Inverse {
            inverse_surface: color!(0x1c1a17),
            inverse_surface_text: color!(0xede4d4),
            inverse_primary: color!(0xcf6229),
        },
        outline: Outline {
            color: color!(0x38322b),
            variant: color!(0x46403a),
        },
        shadow: color!(0x000000),
        scrim: from_argb!(0x66000000),
    }
}
