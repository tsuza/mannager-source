use iced::widget::scrollable::{AutoScroll, Catalog, Rail, Scroller, Status, Style, StyleFn};
use iced::{Background, Border, Shadow, border};

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
    let accent = theme.colors().primary;
    let outline = theme.colors().outline;

    let base_scroller = Scroller {
        background: surface.text_variant.into(),
        border: border::rounded(400),
    };

    let active_rail = Rail {
        background: None,
        scroller: base_scroller,
        border: Border::default(),
    };

    let disabled_rail = Rail {
        background: Some(disabled_container(surface.text).into()),
        scroller: Scroller {
            background: disabled_text(surface.text_variant).into(),
            border: border::rounded(400),
        },
        ..active_rail
    };

    let hovered_scroller = Scroller {
        background: mix(surface.text_variant, accent.color, 0.6).into(),
        border: border::rounded(400),
    };

    let dragged_scroller = Scroller {
        background: accent.color.into(),
        border: border::rounded(400),
    };

    let auto_scroll = AutoScroll {
        background: surface.color.into(),
        border: border::rounded(500).width(1).color(outline.color),
        icon: surface.text_variant,
        shadow: Shadow {
            color: theme.colors().shadow,
            offset: iced::Vector::new(0.0, 6.0),
            blur_radius: 14.0,
        },
    };

    let base = Style {
        container: container::transparent(theme),
        vertical_rail: active_rail,
        horizontal_rail: active_rail,
        gap: None,
        auto_scroll,
    };

    match status {
        Status::Active {
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => Style {
            horizontal_rail: if is_horizontal_scrollbar_disabled {
                disabled_rail.clone()
            } else {
                active_rail.clone()
            },
            vertical_rail: if is_vertical_scrollbar_disabled {
                disabled_rail.clone()
            } else {
                active_rail.clone()
            },
            ..base
        },

        Status::Hovered {
            is_horizontal_scrollbar_hovered,
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => Style {
            horizontal_rail: if is_horizontal_scrollbar_disabled {
                disabled_rail.clone()
            } else if is_horizontal_scrollbar_hovered {
                Rail {
                    scroller: hovered_scroller,
                    ..active_rail
                }
            } else {
                active_rail.clone()
            },
            vertical_rail: if is_vertical_scrollbar_disabled {
                disabled_rail.clone()
            } else if is_vertical_scrollbar_hovered {
                Rail {
                    scroller: hovered_scroller,
                    ..active_rail
                }
            } else {
                active_rail.clone()
            },
            ..base
        },

        Status::Dragged {
            is_horizontal_scrollbar_dragged,
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_disabled,
            is_vertical_scrollbar_disabled,
        } => Style {
            horizontal_rail: if is_horizontal_scrollbar_disabled {
                disabled_rail.clone()
            } else if is_horizontal_scrollbar_dragged {
                Rail {
                    scroller: dragged_scroller,
                    ..active_rail
                }
            } else {
                active_rail.clone()
            },
            vertical_rail: if is_vertical_scrollbar_disabled {
                disabled_rail.clone()
            } else if is_vertical_scrollbar_dragged {
                Rail {
                    scroller: dragged_scroller,
                    ..active_rail
                }
            } else {
                active_rail.clone()
            },
            ..base
        },
    }
}
