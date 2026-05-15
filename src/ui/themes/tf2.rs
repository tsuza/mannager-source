use iced::color;

use super::{
    ColorScheme, Error, Inverse, Outline, Primary, Secondary, Success, Surface, SurfaceContainer,
    Tertiary, from_argb,
};

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
        primary: Primary {
            color: color!(0xcf6229),                // accent-orange
            on_primary: color!(0xf5ede4),           // light cream, readable on orange
            primary_container: color!(0x3d2010),    // deep burnt, behind primary elements
            on_primary_container: color!(0xe08830), // accent-amber, text on container
        },
        secondary: Secondary {
            color: color!(0x96897a),                  // text-secondary
            on_secondary: color!(0x1c1a17),           // bg-base, dark on mid-tone
            secondary_container: color!(0x272320),    // bg-card
            on_secondary_container: color!(0xc4b8a8), // lighter warm grey
        },
        tertiary: Tertiary {
            color: color!(0x8a7a6a),       // warm mid-tone, between secondary and muted
            on_tertiary: color!(0xede4d4), // text-primary
            tertiary_container: color!(0x2a2320), // slightly warm dark
            on_tertiary_container: color!(0xc4b8a8),
        },
        success: Success {
            color: color!(0x4caf50),                // accent-green
            on_success: color!(0x0f1a0f),           // dark green-tinted black
            success_container: color!(0x1a2e1a),    // deep green dark
            on_success_container: color!(0x80d480), // lighter green
        },
        error: Error {
            color: color!(0xc0392b),              // accent-red
            on_error: color!(0x1a0a0a),           // near black
            error_container: color!(0x2a1212),    // deep red dark
            on_error_container: color!(0xe07070), // muted light red
        },
        surface: Surface {
            color: color!(0x1c1a17),              // bg-base
            on_surface: color!(0xede4d4),         // text-primary
            on_surface_variant: color!(0x96897a), // text-secondary
            surface_container: SurfaceContainer {
                lowest: color!(0x1a1815),  // bg-inset
                low: color!(0x1f1c19),     // between inset and card
                base: color!(0x272320),    // bg-card
                high: color!(0x302b28),    // bg-card-hover ish
                highest: color!(0x38322b), // border (used as highest surface)
            },
        },
        inverse: Inverse {
            inverse_surface: color!(0xede4d4),    // text-primary flipped
            inverse_on_surface: color!(0x1c1a17), // bg-base
            inverse_primary: color!(0xcf6229),    // accent-orange
        },
        outline: Outline {
            color: color!(0x38322b),   // border
            variant: color!(0x46403a), // border-light
        },
        shadow: color!(0x000000),
        scrim: from_argb!(0x66000000),
    }
}
