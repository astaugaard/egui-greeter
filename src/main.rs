mod app;
mod auth_thread;
mod inputs;
mod power_area;
mod search_selector;
mod sessions;
mod settings;
mod time_area;

use app::DisplayManager;

use anyhow::anyhow;
use clap::Parser;

use crate::settings::{Args, Settings};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    let settings = Settings::from_args(args)?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 2560.0]),
        ..Default::default()
    };

    auth_thread::Handle::make_handle(settings.user.clone(), move |handle| {
        eframe::run_native(
            "rust display manager",
            options,
            Box::new(|cc| Ok(Box::new(DisplayManager::new(settings, handle, cc)))),
        )
        .map_err(|err| anyhow!("{}", err.to_string()))
    })?;

    Ok(())
}
