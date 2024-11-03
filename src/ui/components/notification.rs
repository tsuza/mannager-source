use notify_rust::{Notification, Timeout};

pub async fn notification(title: &str, body: impl ToString, timeout: impl Into<Timeout>) {
    let mut notification = Notification::new();

    notification
        .appname("MANNager")
        .summary(title)
        .body(&body.to_string())
        .timeout(timeout.into());

    #[cfg(target_os = "linux")]
    notification.icon("org.tsuza.mannager");

    #[cfg(target_os = "windows")]
    notification.app_id("org.tsuza.mannager");

    #[cfg(target_os = "linux")]
    let _ = notification
        .show_async()
        .await
        .and_then(|notification| Ok(notification.on_close(|_| ())));

    #[cfg(target_os = "windows")]
    let _ = notification.show();
}
