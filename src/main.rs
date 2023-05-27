mod errors;
mod pass_scanner;
mod settings;
mod ui;
mod pgp;

use env_logger::Builder;
use iced::{Sandbox, Settings};
use log::LevelFilter;

fn main() -> iced::Result {
    let _log_builder = Builder::from_default_env()
        .filter(Some("passrs"), LevelFilter::Debug)
        .init();

    let mut settings = Settings::default();
    settings.id = Some("PassRS".to_string());
    settings.window.always_on_top = true;

    ui::PassRS::run(settings)
}

#[cfg(test)]
mod test_util;
