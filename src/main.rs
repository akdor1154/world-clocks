// SPDX-License-Identifier: GPL-3.0-only

use app::YourApp;
/// The `app` module is used by convention to indicate the main component of our application.
mod app;
mod config;
mod core;
mod editor;
mod tz;

/// The `cosmic::app::run()` function is the starting point of your application.
/// It takes two arguments:
/// - `settings` is a structure that contains everything relevant with your app's configuration, such as antialiasing, themes, icons, etc...
/// - `()` is the flags that your app needs to use before it starts.
///  If your app does not need any flags, you can pass in `()`.
fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();

    cosmic::applet::run::<YourApp>(())
}
