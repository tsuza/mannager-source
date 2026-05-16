use iced::{
    Length, border, padding,
    widget::{center, container},
};

use crate::ui::{Element, themes::tf2};

pub fn loading<'a, Message: 'a>() -> Element<'a, Message> {
    container(
        center("Loading...")
            .padding(padding::vertical(50).horizontal(100))
            .style(tf2::container::card),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme| tf2::container::main(theme).border(border::width(0)))
    .into()
}
