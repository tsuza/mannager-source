use iced::widget::scrollable::{AutoScroll, Catalog, Rail, Scroller, Status, Style, StyleFn};
use iced::{Background, Border, Shadow, border, color};

use crate::ui::themes::tf2::container;

use super::super::{Theme, disabled_container, disabled_text, mix};

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn default(theme: &Theme, status: Status) -> Style {
    let surface = theme.colors().surface;

    let active = Rail {
        background: None,
        scroller: Scroller {
            background: Background::Color(surface.on_surface_variant),
            border: border::rounded(400),
        },
        border: Border::default(),
    };

    let disabled = Rail {
        background: Some(Background::Color(disabled_container(surface.on_surface))),
        scroller: Scroller {
            background: Background::Color(disabled_text(surface.on_surface_variant)),
            border: border::rounded(400),
        },
        ..active
    };

    let scroll = AutoScroll {
        background: surface.color.into(),
        border: border::rounded(500).width(1).color(surface.color),
        icon: surface.color,
        shadow: Shadow::default(),
    };

    let style = Style {
        container: container::default(theme),
        vertical_rail: active,
        horizontal_rail: active,
        gap: None,
        auto_scroll: scroll,
    };

    match status {
        Status::Active {
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => Style {
            horizontal_rail: if is_horizontal_scrollbar_disabled {
                disabled
            } else {
                active
            },
            vertical_rail: if is_vertical_scrollbar_disabled {
                disabled
            } else {
                active
            },
            ..style
        },
        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => {
            let hovered_rail = Rail {
                scroller: Scroller {
                    background: Background::Color(mix(
                        surface.on_surface_variant,
                        color!(0x994f3f),
                        0.7,
                    )),
                    border: border::rounded(400),
                },
                ..active
            };

            Style {
                horizontal_rail: if is_horizontal_scrollbar_disabled {
                    disabled
                } else if is_horizontal_scrollbar_hovered {
                    hovered_rail
                } else {
                    active
                },
                vertical_rail: if is_vertical_scrollbar_disabled {
                    disabled
                } else if is_vertical_scrollbar_hovered {
                    hovered_rail
                } else {
                    active
                },
                ..style
            }
        }
        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => {
            let dragged_rail = Rail {
                scroller: Scroller {
                    background: Background::Color(color!(0x994f3f)),
                    border: border::rounded(400),
                },
                ..active
            };

            Style {
                horizontal_rail: if is_horizontal_scrollbar_disabled {
                    disabled
                } else if is_horizontal_scrollbar_dragged {
                    dragged_rail
                } else {
                    active
                },
                vertical_rail: if is_vertical_scrollbar_disabled {
                    disabled
                } else if is_vertical_scrollbar_dragged {
                    dragged_rail
                } else {
                    active
                },
                ..style
            }
        }
    }
}
