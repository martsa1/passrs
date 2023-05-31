use std::path::{Path, PathBuf};

use iced::widget::{column, container, scrollable, text, text_input, Text};
use iced::{
    executor, subscription, Application, Command, Element, Event, Length, Subscription, Theme,
};

use super::pass_scanner;
use log::{debug, info, warn};

pub struct PassRS {
    entries: Vec<PathBuf>,
    entry_names: Vec<String>,
    store_path: PathBuf,
    search: String,
    selected: Option<usize>,
    last_search: String,
}

#[derive(Debug, Clone)]
pub enum Action {
    SearchInput(String),
    SelectDown,
    SelectUp,
    SelectEntry,
    EventOccurred(Event),
}

impl Application for PassRS {
    type Executor = executor::Default;
    type Message = Action;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (PassRS, Command<Action>) {
        let store_path = PathBuf::from("/home/sam/.password-store");
        let pass_entries = pass_scanner::collect_pass_files(&store_path);

        match pass_entries {
            Ok(entries) => {
                info!("Found {} password entries.", entries.len());
                let entry_names = entries
                    .iter()
                    .filter_map(|i| entry_to_ui_format(&i, &store_path))
                    .collect();

                (
                    Self {
                        entries,
                        entry_names,
                        store_path,
                        search: "".to_string(),
                        selected: None,
                        last_search: "".to_string(),
                    },
                    Command::none(),
                )
            }
            Err(err) => {
                warn!("Failed to retrieve store path entries: '{}'", err);

                (
                    Self {
                        entries: vec![],
                        entry_names: vec![],
                        store_path,
                        search: "".to_string(),
                        selected: None,
                        last_search: "".to_string(),
                    },
                    Command::none(),
                )
            }
        }
    }

    fn title(&self) -> String {
        "PassRS".into()
    }

    fn update(&mut self, message: Action) -> Command<Action> {
        match message {
            Action::SearchInput(input) => {
                info!("SearchInput triggered: '{}'.", input);
                self.last_search = self.search.clone();
                self.search = input;

                // If search hasn't changed, don't recompute the list - but do if its the first time.
                debug!(
                    "Search: {:?}, last search: {:?}",
                    self.search, self.last_search
                );
                if self.search != self.last_search {
                    let entry_strs = self
                        .entries
                        .iter()
                        .filter_map(|i| entry_to_ui_format(&i, &self.store_path))
                        .collect();
                    self.entry_names = pass_scanner::filter_pass_entries(&entry_strs, &self.search)
                        .unwrap_or(vec![]);
                }
            }
            Action::SelectUp => match self.selected {
                Some(idx) => {
                    self.selected = if idx > 0 {
                        Some(idx - 1)
                    } else {
                        Some(self.entries.len())
                    }
                }
                None => {}
            },
            Action::SelectDown => {
                match self.selected {
                    Some(idx) => {
                        self.selected = if idx + 1 == self.entries.len() {
                            // Roll from bottom to top of list
                            Some(0)
                        } else {
                            Some(idx + 1)
                        };
                    }
                    None => {
                        self.selected = Some(0);
                    }
                }
            }
            Action::SelectEntry => {
                let id = match self.selected {
                    Some(idx) => idx,
                    None => 0,
                };

                let entry_name = &self.entry_names[id];
                let entry = match entry_from_ui_format(&entry_name, &self.store_path) {
                    Some(ent) => ent,
                    None => {
                        warn!("Failed to resolve selected path entry: {}", entry_name);
                        return Command::none();
                    }
                };

                info!("Selecting entry {}!", entry.to_string_lossy());
            }
            Action::EventOccurred(_event) => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<Action> {
        let mut dark_row = iced::widget::container::Appearance::default();
        dark_row.background = Some(iced::color!(10, 10, 10).into());

        let entry_names = render_pass_entries(&self.entry_names, &self.store_path);
        let entry_names: Vec<iced::widget::Container<Action, iced::Renderer>> = entry_names
            .into_iter()
            .enumerate()
            .map(|(idx, entry)| {
                if let Some(selection_index) = self.selected {
                    if selection_index == idx {
                        // Make a... dark-themed container..?
                        let container = container(entry);
                        return container.style(iced::theme::Container::Box);
                    }
                }
                // Make a normally-themed container.
                return container(entry);
            })
            .collect();

        let scroll_box = scrollable(column(entry_names.into_iter().map(|i| i.into()).collect()))
            .width(Length::Fill);
        let search_box = text_input("Search...", &self.search)
            .on_input(Action::SearchInput)
            .padding(2);

        column![search_box, scroll_box]
            .spacing(2)
            .width(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Action> {
        // TODO: Filter to keyboard enter etc...
        subscription::events_with(|event, status| {
            match event {
                iced::Event::Keyboard(key) => {
                    //debug!("received Keyboard event: {:?}", key);
                    match key {
                        //iced::keyboard::Event::KeyPressed {
                        //    key_code,
                        //    modifiers,
                        //} => {
                        //    debug!("received Key Press: {:?}|{:?}", key_code, modifiers);
                        //    None
                        //}
                        iced::keyboard::Event::KeyReleased {
                            key_code,
                            modifiers: _,
                        } => match key_code {
                            iced::keyboard::KeyCode::Enter => Some(Action::SelectEntry),
                            iced::keyboard::KeyCode::Down => Some(Action::SelectDown),
                            iced::keyboard::KeyCode::Up => Some(Action::SelectUp),
                            _ => None,
                        },
                        //iced::keyboard::Event::ModifiersChanged(modifiers) => {
                        //    debug!("Modifiers changed: {:?}", modifiers);
                        //    None
                        //}
                        _ => None,
                    }
                }
                iced::Event::Window(win) => {
                    debug!("received Window event: {:?}", win);
                    None
                }
                _ => None, // Ignore mouse events for now.
            }
        })
    }
}

fn entry_to_ui_format(entry: &PathBuf, base_path: &Path) -> Option<String> {
    let rel_path = entry.strip_prefix(&base_path);

    match rel_path {
        Ok(p) => {
            let entry_name = p.to_string_lossy().replace(".gpg", "");
            Some(entry_name)
        }
        Err(_) => None,
    }
}

fn entry_from_ui_format(entry_name: &str, base_path: &Path) -> Option<PathBuf> {
    let mut rel_path = base_path.to_owned();

    // Entry name may have a fuzzy match prefix on it...
    let mut entry = entry_name;
    entry = match entry.split_once(": ") {
        Some((_, ent)) => ent,
        None => entry,
    };

    rel_path.push(entry);
    rel_path.set_extension("gpg");


    if rel_path.exists() && rel_path.is_file() {
        return Some(rel_path);
    }
    None
}

fn render_pass_entries<'a>(entries: &Vec<String>, base_path: &Path) -> Vec<Text<'a>> {
    let matches = entries.iter().map(|i| text(i.to_owned())).collect();

    matches
}
