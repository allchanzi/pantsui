mod app;
mod entities;
mod error;
mod features;
mod leader;
mod shared;
mod state;
mod tui;
mod ui;

use crate::{app::App, error::AppResult};

fn main() -> AppResult<()> {
    let root = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });
    tui::run(App::new(root))
}
