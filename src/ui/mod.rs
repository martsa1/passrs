use std::path::{Path, PathBuf};

use iced::widget::{column, container, row, scrollable, text, text_input, Text, TextInput};
use iced::{Element, Length, Sandbox};

use super::pass_scanner;
use log::{info, warn};

pub struct PassRS {
    entries: Vec<PathBuf>,
    store_path: PathBuf,
    search: String,
}

#[derive(Debug, Clone)]
pub enum Action {
    RefreshList,
    ClearList,
    SearchInput(String),
}

impl Sandbox for PassRS {
    type Message = Action;

    fn new() -> PassRS {
        let store_path = PathBuf::from("/home/sam/.password-store");
        let pass_entries = pass_scanner::collect_pass_files(&store_path);

        match pass_entries {
            Ok(entries) => {
                info!("Found {} password entries.", entries.len());
                Self {
                    entries,
                    store_path,
                    search: "".to_string(),
                }
            }
            Err(err) => {
                warn!("Failed to retrieve store path entries: '{}'", err);

                Self {
                    entries: vec![],
                    store_path,
                    search: "".to_string(),
                }
            }
        }
    }

    fn title(&self) -> String {
        "PassRS".into()
    }

    fn update(&mut self, message: Action) {
        match message {
            Action::RefreshList => {
                self.entries = pass_scanner::collect_pass_files(&self.store_path).unwrap();
            }
            Action::ClearList => {
                self.entries.clear();
            }
            Action::SearchInput(input) => {
                info!("SearchInput triggered: '{}'.", input);
                self.search = input;
            }
        }
    }

    fn view(&self) -> Element<Action> {
        let mut entries: Vec<String> = self.entries.iter().filter_map(|i| entry_to_ui_format(&i, &self.store_path)).collect();
        entries = pass_scanner::filter_pass_entries(&entries, &self.search).unwrap_or(vec![]);

        let entry_names = render_pass_entries(&entries, &self.store_path);

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

fn render_pass_entries<'a>(entries: &Vec<String>, base_path: &Path) -> Vec<Text<'a>> {
    let matches = entries
        .iter()
        .map(|i| text(i.to_owned()))
        .collect();

    matches
}
