mod errors;
mod pass_scanner;
mod settings;
mod ui;
mod pgp;

use env_logger::Builder;
use iced::{Sandbox, Settings};
use log::LevelFilter;

fn main() -> iced::Result {
    let log_builder = Builder::from_default_env()
        .filter(Some("passrs"), LevelFilter::Debug)
        .init();

    ui::PassRS::run(Settings::default())
}
