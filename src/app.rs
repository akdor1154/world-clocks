// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

use chrono::DurationRound;
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{ConfigGet, ConfigSet, CosmicConfigEntry};
use cosmic::iced::futures::SinkExt;
use cosmic::iced::{self, stream, window, Alignment, Length, Limits, Subscription};
use cosmic::iced_widget::Row;
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::widget::{self, autosize, horizontal_space, settings, vertical_space};
use cosmic::{cosmic_config, Application, Element};
use tokio::time;

use crate::{config, editor, fl};
use crate::config::WorldClocksConfig;
use anyhow::{Result, Context};

struct Tz {
    #[allow(dead_code)]
    name: String,
    display_name: String,
    tz: tzfile::Tz,
}

impl Tz {
    fn from_names(name: &str, display_name: &str) -> Result<Tz> {
        let tz = tzfile::Tz::named(name).context(format!("Couldn\'t load timezone {}", name))?;
        Ok(Tz {
            name: name.to_owned(),
            display_name: display_name.to_owned(),
            tz,
        })
    }

    fn from_name(name: &str) -> Result<Tz> {
        let display_name = name.rsplitn(2, "/").next().unwrap().to_owned();
        return Self::from_names(name, &display_name)
    }

}

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
// #[derive(Default)]
pub struct YourApp {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The popup id.
    popup: Option<window::Id>,
    editor: editor::Editor,
    /// Example row toggler.
    // example_row: bool,
    now: chrono::DateTime<chrono::Utc>,
    // config
    config: cosmic_config::Config,
    timezones: Vec<Result<Tz>>,
}

static AUTOSIZE_MAIN_ID: LazyLock<widget::Id> = LazyLock::new(|| widget::Id::new("autosize-main"));
/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(window::Id),
    // ToggleExampleRow(bool),
    Tick,
    ConfigChanged(WorldClocksConfig),
    Editor(editor::Message)
}

impl From<editor::Message> for Message {
    fn from(msg: editor::Message) -> Self {
        Self::Editor(msg)
    }
}

impl YourApp {
    fn tzs_from_config(c: &WorldClocksConfig) -> Vec<Result<Tz>> {
        return c.timezones.iter().map(|tz| { Tz::from_names(&tz.name, &tz.display_name)}).collect()
    }
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
        let cconfig = cosmic_config::Config::new(YourApp::APP_ID, 1).unwrap();
        // TODO: try to see if local config doesn't exist yet, if so write the default.

        // let mut config = match WorldClocksConfig::get_entry(&cconfig) {
        //     Ok(config) => config,
        //     Err((e, config)) => {
        //         println!("{:#?}", e); // TODO!!
        //         config
        //     }
        // };
        let config = WorldClocksConfig::default();

        let timezones = YourApp::tzs_from_config(&config);


        let app = YourApp {
            core,
            now: chrono::Utc::now(),
            config: cconfig,
            timezones: timezones,
            popup: None,
            editor: editor::Editor::default()
            // ..Default::default()
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


        let texts = self.timezones.iter().map(|rtz| {
            let Ok(tz) = rtz else {
                return Element::from(self.core.applet.text("Error!"))
            };
            let time_str = self.now.with_timezone(&&tz.tz).format("%H:%M");
            let s = format!("{} {}", time_str, tz.display_name);
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
        // let content_list = widget::list_column()
        //     .padding(5)
        //     .spacing(0)
        //     .add(settings::item(
        //         fl!("example-row"),
        //         widget::toggler(self.example_row).on_toggle(Message::ToggleExampleRow),
        //     ));
        // self.core.applet.popup_container(content_list).into()

        self.core.applet.popup_container(self.editor.view().map(Message::Editor)).into()
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
            Message::ConfigChanged(c) => {
                self.timezones = YourApp::tzs_from_config(&c)
            }
            Message::Tick => {
                self.now = chrono::Utc::now();
            }
            Message::Editor(msg) => {
                match self.editor.update(msg) {
                    None => {},
                    Some(editor::Output::NewConfig(c)) => {
                        c.write_entry(&self.config).unwrap();
                    }
                };
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

        let config_subscription = self.core.watch_config(Self::APP_ID).map(|u| {
            for err in u.errors {
                tracing::error!(?err, "Error watching config");
            }
            Message::ConfigChanged(u.config)
        });

        Subscription::batch(vec![time_subscription(), config_subscription])
    }
}
