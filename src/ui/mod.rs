use std::path::{PathBuf, Path};

use iced::widget::{column, text, text_input};
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
        let entry_names = render_pass_entries(&self.entries, &self.store_path);

        let mut entry_text = String::new();
        match entry_names {
            Some(entries) => {
                entry_text = entries.join("\n");
            }
            None => {}
        }

        let pass_text = text(entry_text);
        let search_box = text_input("Search...", &self.search).on_input(Action::SearchInput).padding(2);

        column![search_box, pass_text]
            .spacing(2)
            .width(Length::Fill)
            .into()
    }
}

fn render_pass_entries(entries: &Vec<PathBuf>, base_path: &Path) -> Option<Vec<String>> {
    entries
        .iter()
        .map(|i| {
            let rel_path = i.strip_prefix(&base_path);
            match rel_path {
                Ok(p) => {
                    let entry_name = p.to_string_lossy().to_string().replace(".gpg", "");
                    Some(entry_name)
                }
                Err(_) => None,
            }
        })
        .collect()
}
