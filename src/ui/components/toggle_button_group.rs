use iced::{
    border::radius,
    widget::{Row, Text, button},
};

use crate::ui::{Element, themes::Theme};

pub fn grouped_buttons<'a, Value, Message>(
    items: impl IntoIterator<Item = (Text<'a, Theme>, Value)>,
    active: Value,
    on_press: impl Fn(Value) -> Message + Clone + 'a,
    style: impl Fn(&Theme, button::Status) -> button::Style + Clone + 'a,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
    Value: PartialEq,
{
    let items: Vec<_> = items.into_iter().collect();
    let last_index = items.len().saturating_sub(1);

    let buttons =
        items
            .into_iter()
            .enumerate()
            .map(|(index, (label, mode))| -> Element<'a, Message> {
                let is_active = mode == active;

                let status = if is_active {
                    button::Status::Pressed
                } else {
                    button::Status::Active
                };

                let style = style.clone();

                let style = move |theme: &Theme, _: button::Status| {
                    let mut default = style(theme, status);

                    if index == 0 {
                        default.border.radius = default.border.radius.right(0);
                    } else if index == last_index {
                        default.border.radius = default.border.radius.left(0);
                    } else {
                        default.border.radius = radius(0);
                    }

                    default
                };

                let msg = on_press.clone()(mode);

                button(label).on_press(msg).style(style).into()
            });

    Row::with_children(buttons).into()
}
