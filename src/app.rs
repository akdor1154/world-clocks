// SPDX-License-Identifier: GPL-3.0-only

use chrono::DurationRound;
use cosmic::app::{Core, Task};
use cosmic::iced::border::width;
use cosmic::iced::futures::SinkExt;
use cosmic::iced::{stream, window, Alignment, Length, Limits, Subscription};
use cosmic::iced_widget::{row, Row};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self, autosize, horizontal_space, settings, vertical_space};
use cosmic::{Application, Element};
use itertools::Itertools;
use once_cell::sync::Lazy;
use tokio::time;

use crate::fl;

struct Tz {
    fullname: String,
    shortname: String,
    tz: tzfile::Tz,
}

impl Tz {
    fn from_name(name: &str) -> Option<Tz> {
        let Ok(tz) = tzfile::Tz::named(name) else {
            return None;
        };
        let shortname = name.rsplitn(2, "/").next().unwrap().to_owned();
        Tz {
            fullname: name.to_owned(),
            shortname: shortname,
            tz,
        }
        .into()
    }
}

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
#[derive(Default)]
pub struct YourApp {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The popup id.
    popup: Option<window::Id>,
    /// Example row toggler.
    example_row: bool,
    now: chrono::DateTime<chrono::Utc>,
    tzs: Vec<Tz>,
}

static AUTOSIZE_MAIN_ID: Lazy<widget::Id> = Lazy::new(|| widget::Id::new("autosize-main"));
/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(window::Id),
    ToggleExampleRow(bool),
    Tick,
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for YourApp {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "it.jmwh.WorldClocks";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = YourApp {
            core,
            now: chrono::Utc::now(),
            tzs: vec![
                Tz::from_name("UTC").unwrap(),
                Tz::from_name("Europe/London").unwrap(),
                Tz::from_name("Australia/Perth").unwrap(),
            ],
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        let texts = self.tzs.iter().map(|tz| {
            let time_str = self.now.with_timezone(&&tz.tz).format("%H:%M");
            let s = format!("{} {}", time_str, tz.shortname);
            Element::from(self.core.applet.text(s))
        });

        let pad = Length::Fixed(self.core.applet.suggested_padding(true).into());
        let height =
            self.core.applet.suggested_size(true).1 + 2 * self.core.applet.suggested_padding(true);
        let vspacer = vertical_space().height(Length::Fixed(height.into()));

        let elems =
            itertools::intersperse_with(texts, || Element::from(horizontal_space().width(pad)));
        let content = Row::from_iter(elems)
            .push(vspacer)
            .align_y(Alignment::Center);

        let button = cosmic::widget::button::custom(content)
            .padding([0, self.core.applet.suggested_padding(true)])
            .class(cosmic::theme::Button::AppletMenu)
            .on_press_down(Message::TogglePopup);

        let autosize = autosize::autosize(button, AUTOSIZE_MAIN_ID.clone());
        autosize.into()
    }

    fn view_window(&self, _id: window::Id) -> Element<Self::Message> {
        let content_list = widget::list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                fl!("example-row"),
                widget::toggler(self.example_row).on_toggle(Message::ToggleExampleRow),
            ));

        self.core.applet.popup_container(content_list).into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = window::Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ToggleExampleRow(toggled) => self.example_row = toggled,
            Message::Tick => {
                self.now = chrono::Utc::now();
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        fn time_subscription() -> Subscription<Message> {
            Subscription::run_with_id(
                "time_sub",
                stream::channel(1, async move |mut output| {
                    let mut timer = time::interval(time::Duration::from_secs(60));

                    loop {
                        timer.tick().await;
                        let _ = output.send(Message::Tick).await;

                        let now = chrono::Utc::now();
                        let next_minute_dt = now
                            .duration_round_up(chrono::TimeDelta::minutes(1))
                            .unwrap();
                        let diff = time::Duration::from_millis(
                            (next_minute_dt - now).num_milliseconds().max(0) as u64,
                        );
                        timer.reset_after(diff);
                    }
                }),
            )
        }

        Subscription::batch(vec![time_subscription()])
    }
}
