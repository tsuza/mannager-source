use iced::color;

use super::{
    ColorScheme, Error, Inverse, Outline, Primary, Secondary, Surface, SurfaceContainer, Tertiary,
    from_argb,
};

pub mod button;
pub mod container;
pub mod scrollable;
pub mod text_input;

pub const fn color_scheme() -> ColorScheme {
    ColorScheme {
        primary: Primary {
            color: color!(0xc49a6c),
            on_primary: color!(0x1c1a19),
            primary_container: color!(0x9c7a55),
            on_primary_container: color!(0xFFFFFF),
        },
        secondary: Secondary {
            color: color!(0x645c51),
            on_secondary: color!(0xFFFFFF),
            secondary_container: color!(0x7e7366),
            on_secondary_container: color!(0xeee5cf),
        },
        tertiary: Tertiary {
            color: color!(0x6b8c77),
            on_tertiary: color!(0xFFFFFF),
            tertiary_container: color!(0x527061),
            on_tertiary_container: color!(0xeae9e9),
        },
        error: Error {
            color: color!(0xa93131),
            on_error: color!(0xFFFFFF),
            error_container: color!(0xba5a5a),
            on_error_container: color!(0xFFFFFF),
        },
        surface: Surface {
            color: color!(0x1c1a19),
            on_surface: color!(0xeae9e9),
            on_surface_variant: color!(0xFFFFFF),
            surface_container: SurfaceContainer {
                lowest: color!(0x2A2725),
                low: color!(0x3f3d3b),
                base: color!(0x555251),
                high: color!(0x6a6866),
                highest: color!(0x7f7d7c),
            },
        },
        inverse: Inverse {
            inverse_surface: color!(0xeae9e9),
            inverse_on_surface: color!(0x1c1a19),
            inverse_primary: color!(0xc49a6c),
        },

        outline: Outline {
            color: color!(0x8a857f),
            variant: color!(0x6a6661),
        },
        shadow: color!(0x000000),
        scrim: from_argb!(0x4d000000),
    }
}
