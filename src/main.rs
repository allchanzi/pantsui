mod app;
mod entities;
mod error;
mod features;
mod shared;
mod state;
mod tui;
mod ui;

use crate::{app::App, error::AppResult};

fn main() -> AppResult<()> {
    tui::run(App::new())
}
