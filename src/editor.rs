use cosmic::{
    cosmic_config::{self, CosmicConfigEntry},
    iced::{Alignment, Length},
    widget::{self, combo_box, icon},
    Element,
};

use crate::{
    config::{Tz, WorldClocksConfig},
    tz::{ValidTz, TZ_NAMES},
};
use anyhow::Result;

pub struct Editor {
    text_input_buf: String,
    tz_input_state: widget::combo_box::State<String>,
    tz_input_buf: Option<String>,
    text_input_ids: Vec<(widget::Id, widget::Id)>,
    editing_item: Option<(usize, ItemEditState)>,
    tzs: Vec<MaybeTz>,
    app_config: cosmic_config::Config,
}

#[derive(Debug, Clone)]
pub enum ItemEditState {
    Name,
    DisplayName,
}

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub enum EditList {
    NewConfig(WorldClocksConfig),
    Reorder(Direction, usize),
    AddAtEnd(),
    Remove(usize),
}

#[derive(Debug, Clone)]
pub enum EditItem {
    SetDisplayName(String),
    SetTz(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    EditList(EditList),
    EditItem(usize, EditItem),
    StartEditing(usize, ItemEditState),
    CancelEditing,
    Input(String),
}

impl From<EditList> for Message {
    fn from(value: EditList) -> Self {
        return Message::EditList(value);
    }
}
impl From<(usize, EditItem)> for Message {
    fn from((i, value): (usize, EditItem)) -> Self {
        return Message::EditItem(i, value);
    }
}

#[derive(Debug, Clone)]
pub enum Output {
    // NewConfig(WorldClocksConfig)
}
type MaybeTz = Result<Tz, (Tz, anyhow::Error)>;

impl Editor {
    pub fn new(app_id: &str) -> Self {
        let app_config = cosmic_config::Config::new(app_id, 1).unwrap();
        let initial_config = WorldClocksConfig::get_entry(&app_config).unwrap_or_default();
        let tzs: Vec<MaybeTz> = initial_config.timezones.into_iter().map(validate).collect();
        let n = tzs.len();
        return Editor {
            editing_item: None,
            tzs: tzs,
            text_input_ids: (0..n)
                .map(|i| {
                    (
                        widget::Id::new(format!("input-full-{i}")),
                        widget::Id::new(format!("input-display-{i}")),
                    )
                })
                .collect(),
            text_input_buf: String::new(),
            tz_input_state: combo_box::State::new(TZ_NAMES.to_vec()),
            tz_input_buf: None,
            app_config,
        };
    }
    pub(super) fn view(&self) -> cosmic::Element<Message> {
        // for each tz, draw a row with
        // Name
        // Full Name

        let cosmic::cosmic_theme::Spacing { space_xs, .. } =
            cosmic::theme::active().cosmic().spacing;

        let mut content_list = widget::list_column().padding(5).spacing(0).add(
            widget::row::with_children(vec![
                widget::button::icon(icon::from_name("list-add-symbolic"))
                    .label("Add")
                    .on_press(EditList::AddAtEnd().into())
                    .into(),
                widget::horizontal_space().into(),
                widget::button::icon(icon::from_name("document-revert-symbolic"))
                    .label("Reset")
                    .on_press(EditList::NewConfig(WorldClocksConfig::default()).into())
                    .into(),
            ])
            .align_y(Alignment::Center),
        );

        let (editing_i, item_edit_state) = match &self.editing_item {
            Some((i, s)) => (*i, Some(s)),
            None => (usize::MAX, None),
        };
        for (i, tz) in self.tzs.iter().enumerate() {
            let edit_state = if editing_i == i {
                item_edit_state
            } else {
                None
            };

            content_list = content_list.add(
                // I looked at the Panel Settings -> Applet list to see how to do a DnD list,
                // and it seems highly DIY.. up/down buttons it is for now, I don't want to write DnD handling from scratch.
                widget::row::with_children(vec![
                    widget::column()
                        .push(
                            widget::button::icon(icon::from_name("go-up-symbolic"))
                                .extra_small()
                                .on_press(EditList::Reorder(Direction::Up, i).into()),
                        )
                        .push(
                            widget::button::icon(icon::from_name("go-down-symbolic"))
                                .extra_small()
                                .on_press(EditList::Reorder(Direction::Down, i).into()),
                        )
                        .into(),
                    self.tz_list_item(i, tz, edit_state)
                        .width(Length::Fill)
                        .into(),
                    widget::button::icon(icon::from_name("list-remove-symbolic"))
                        .extra_small()
                        .on_press(EditList::Remove(i).into())
                        .into(),
                ])
                .spacing(space_xs)
                .align_y(Alignment::Center),
            )
        }
        return content_list.into();
    }

    pub(super) fn update(&mut self, msg: Message) -> Option<Output> {
        match msg {
            Message::EditList(el) => {
                match el {
                    EditList::Reorder(dir, i) => {
                        let j = match dir {
                            Direction::Up => {
                                if i <= 0 {
                                    return None;
                                };
                                i - 1
                            }
                            Direction::Down => {
                                if self.tzs.len() - 1 <= i {
                                    return None;
                                };
                                i + 1
                            }
                        };
                        self.tzs.swap(i, j);
                    }

                    EditList::AddAtEnd() => {
                        let new_tz: MaybeTz = Err((
                            Tz {
                                display_name: "Mordor".to_owned(),
                                name: "Middle_Earth/Mordor".to_owned(),
                            },
                            anyhow::anyhow!("please choose a timezone"),
                        ));
                        let i = self.tzs.len();
                        self.tzs.push(new_tz);
                        self.text_input_ids.push((
                            widget::Id::new(format!("input-full-{i}")),
                            widget::Id::new(format!("input-display-{i}")),
                        ))
                    }

                    EditList::Remove(i) => {
                        let _ = self.tzs.remove(i);
                        let _ = self.text_input_ids.pop();
                    }

                    EditList::NewConfig(c) => {
                        self.tzs = c.timezones.into_iter().map(validate).collect();
                    }
                }
                self.maybe_update_config();
                return None;
            }

            Message::EditItem(i, ei) => {
                let mut_tz = match self.tzs.get_mut(i) {
                    Some(Ok(_t)) => _t,
                    Some(Err((_t, _e))) => _t,
                    None => return None,
                };
                'update_name: {
                    match ei {
                        EditItem::SetDisplayName(new_display_name) => {
                            mut_tz.display_name = new_display_name;
                        }
                        EditItem::SetTz(new_name) => {
                            // if the user chooses the same TZ, don't reset the display name.
                            if new_name == mut_tz.name {
                                break 'update_name;
                            }
                            let new_tz = tz_from_name(new_name);
                            let _ = std::mem::replace(&mut self.tzs[i], new_tz);
                        }
                    }
                }
                self.editing_item = None;
                self.maybe_update_config();
                return None;
            }

            Message::StartEditing(i, ie) => {
                let tz = match self.tzs.get(i) {
                    Some(Ok(tz)) => tz,
                    Some(Err((tz, _))) => tz,
                    None => return None,
                };
                match ie {
                    ItemEditState::DisplayName => {
                        self.text_input_buf = tz.display_name.to_owned();
                    }
                    ItemEditState::Name => {
                        self.tz_input_buf = Some(tz.name.to_owned());
                    }
                };
                self.editing_item = Some((i, ie));
                return None;
            }

            Message::CancelEditing => {
                self.editing_item = None;
                return None;
            }

            Message::Input(s) => {
                self.text_input_buf = s;
                return None;
            }
        }
    }

    fn maybe_update_config(&self) {
        let maybe_tzs: Option<Vec<Tz>> = self
            .tzs
            .iter()
            .map(|r| r.as_ref().ok())
            .map(|tz| tz.map(|tz| tz.clone()))
            .collect();
        match maybe_tzs {
            Some(tzs) => WorldClocksConfig { timezones: tzs }
                .write_entry(&self.app_config)
                .unwrap(),
            None => {}
        }
    }

    fn tz_list_item<'a>(
        &'a self,
        i: usize,
        maybe_tz: &'a MaybeTz,
        editing: Option<&ItemEditState>,
    ) -> cosmic::widget::Column<'a, Message> {
        let (tz, err) = maybetz_to_option(maybe_tz);

        let display_name_widget: Element<_> = if let Some(&ItemEditState::DisplayName) = editing {
            widget::inline_input("Display Name", &self.text_input_buf)
                .id(self.text_input_ids[i].0.clone())
                .editing(true)
                .on_input(Message::Input)
                .on_unfocus(Message::CancelEditing)
                .on_submit(move |s| Message::EditItem(i, EditItem::SetDisplayName(s)))
                .into()
        } else {
            widget::button::text(&tz.display_name)
                .on_press(Message::StartEditing(i, ItemEditState::DisplayName))
                .into()
        };

        let tz_name_widget: Element<_> = if let Some(&ItemEditState::Name) = editing {
            widget::combo_box(
                &self.tz_input_state,
                "Timezone Name",
                self.tz_input_buf.as_ref(),
                move |s| Message::EditItem(i, EditItem::SetTz(s)),
            )
            .on_close(Message::CancelEditing)
            .into()
        } else {
            widget::button::custom(widget::text::caption(&tz.name))
                .class(cosmic::theme::Button::Text)
                .on_press(Message::StartEditing(i, ItemEditState::Name))
                .into()
        };

        return widget::column()
            .push(display_name_widget)
            // .push(widget::text::caption(&tz.name))
            .push(tz_name_widget)
            .push_maybe(err.map(|e| widget::text::text(e.to_string())))
            .into();
    }
}

fn maybetz_to_option(maybe_tz: &MaybeTz) -> (&Tz, Option<&anyhow::Error>) {
    match maybe_tz {
        Ok(tz) => {
            return (tz, None);
        }
        Err((tz, e)) => {
            return (tz, Some(e));
        }
    }
}

fn validate(tz: Tz) -> MaybeTz {
    match ValidTz::from_names(&tz.name, &tz.display_name) {
        Ok(_) => return Ok(tz),
        Err(e) => return Err((tz, e)),
    };
}

fn tz_from_name(name: String) -> MaybeTz {
    let display_name = name.rsplitn(2, "/").next().unwrap().replace("_", " ");
    return validate(Tz {
        name: name,
        display_name: display_name,
    });
}
