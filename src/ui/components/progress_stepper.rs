use iced::{
    Alignment, border,
    widget::{Row, Text, container, row, rule, text},
};

use crate::{
    icon,
    ui::{
        Element,
        themes::{Theme, tf2},
    },
};

// TODO: Look into impl IntoFragment<'a>
pub fn stepper<'a, Value, Message>(
    items: impl IntoIterator<Item = (&'a str, Value)>,
    active: Value,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    Value: PartialEq + Ord,
{
    let items = items.into_iter().enumerate().map(
        |(index, (item, value))| -> (Element<'a, Message>, Value) {
            let label = if value < active {
                icon::check()
            } else {
                text(index + 1)
            };

            let container_style = if value <= active {
                |theme: &Theme| container::Style {
                    background: Some(theme.colors().primary.color.into()),
                    border: border::rounded(f32::INFINITY),
                    ..Default::default()
                }
            } else {
                |theme: &Theme| {
                    let base = tf2::container::outlined(theme);

                    container::Style {
                        border: base.border.rounded(f32::INFINITY),
                        ..base
                    }
                }
            };

            let label_style = if value < active {
                |theme: &Theme| tf2::text::muted(theme)
            } else if value == active {
                |theme: &Theme| tf2::text::default(theme)
            } else {
                |theme: &Theme| tf2::text::muted(theme)
            };

            (
                step_label(label, item.into(), container_style, label_style),
                value,
            )
        },
    );

    let items = items
        .flat_map(|(element, value)| {
            let rule_style = if value <= active {
                |theme: &Theme| rule::Style {
                    color: theme.colors().primary.color,
                    ..tf2::rule::primary(theme)
                }
            } else {
                tf2::rule::full_width
            };

            [rule::horizontal(1).style(rule_style).into(), element]
        })
        .skip(1);

    Row::with_children(items)
        .align_y(Alignment::Center)
        .spacing(10)
        .into()
}

fn step_label<'a, Message>(
    number: Text<'a, Theme>,
    label: Text<'a, Theme>,
    container_style: impl Fn(&Theme) -> container::Style + Clone + 'a,
    text_style: impl Fn(&Theme) -> text::Style + Clone + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    row![
        container(
            number
                .size(11)
                .line_height(1.0)
                .width(20)
                .height(20)
                .center()
                .style(text_style.clone()),
        )
        .align_y(Alignment::Center)
        .align_x(Alignment::Center)
        .style(container_style),
        label.size(13).style(text_style),
    ]
    .align_y(Alignment::Center)
    .spacing(8)
    .into()
}
