use iced::{
    Alignment, Font, Length, Shadow, padding,
    widget::{button, column, container, markdown, row, scrollable, space, text},
};
use velopack::{UpdateCheck, UpdateInfo, UpdateManager};

use crate::{
    UPDATES_URL, icon,
    ui::{
        Element, Message,
        themes::{Theme, tf2},
    },
};

pub fn update_dialog<'a>(
    patch_notes: &'a markdown::Content,
    info: Option<&'a UpdateInfo>,
) -> Element<'a, Message> {
    let Some(info) = info else {
        return space().into();
    };

    let header = {
        let close_button = button(
            icon::close()
                .width(Length::Fill)
                .height(Length::Fill)
                .size(20)
                .center(),
        )
        .on_press(Message::DialogClose)
        .width(Length::Shrink)
        .height(Length::Shrink);

        let title = column![
            text("Update Available")
                .width(Length::Fill)
                .size(30)
                .font(Font {
                    weight: iced::font::Weight::Bold,
                    ..Font::new("TF2 Build")
                }),
            text("A new version of MANNager is ready to install")
                .size(12)
                .style(tf2::text::muted)
        ]
        .width(Length::Fill)
        .spacing(5);

        row![title, close_button]
            .height(100)
            .align_y(Alignment::Center)
            .padding(padding::vertical(20).horizontal(22))
    };

    let body = {
        let new_version = &info.TargetFullRelease.Version;

        let download_size = &info.TargetFullRelease.Size / (1024 * 1024);

        let update_info = container(
            row![
                container(text!("v{}", env!("CARGO_PKG_VERSION")).size(15))
                    .padding(padding::horizontal(10).vertical(6))
                    .style(tf2::container::info_container),
                icon::move_right(),
                container(text!("v{}", new_version).size(15))
                    .padding(padding::horizontal(10).vertical(6))
                    .style(tf2::container::primary),
                space::horizontal(),
                text!("{download_size} MBs")
                    .size(10)
                    .style(tf2::text::muted)
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .padding(padding::vertical(14).horizontal(22)),
        )
        .style(|theme| {
            tf2::container::base(theme)
                .shadow(Shadow::default())
                .border(tf2::container::base(theme).border.rounded(0))
        });

        let update_patch_notes = container(
            scrollable(
                markdown(
                    patch_notes.items(),
                    markdown::Settings::with_style(Theme::TeamFortress2),
                )
                .map(Message::LinkClicked),
            )
            .auto_scroll(true)
            .width(Length::Fill)
            .spacing(5),
        )
        .padding(padding::vertical(14).horizontal(22));

        column![update_info, update_patch_notes].height(Length::Fill)
    };

    let footer = row![
        button("Not now")
            .on_press(Message::DialogClose)
            .padding(padding::vertical(10).horizontal(20)),
        space::horizontal(),
        button("Download")
            .on_press(Message::UpdateApp)
            .padding(padding::vertical(10).horizontal(20))
            .style(tf2::button::primary)
    ]
    .width(Length::Fill)
    .padding(padding::vertical(14).horizontal(22))
    .align_y(Alignment::Center);

    container(column![header, body, footer])
        .width(600)
        .height(500)
        .into()
}

pub async fn check_for_updates() -> Result<(UpdateManager, UpdateCheck), velopack::Error> {
    use velopack::*;

    let source = sources::HttpSource::new(UPDATES_URL);

    let um = UpdateManager::new(source, None, None)?;

    let update_check = um.check_for_updates_async().await?;

    Ok((um, update_check))
}

pub async fn update_app(um: UpdateManager, updates: UpdateInfo) -> Result<(), velopack::Error> {
    um.download_updates_async(&updates, None).await?;

    um.wait_exit_then_apply_updates(&updates, false, false, Vec::<String>::new())?;

    Ok(())
}
