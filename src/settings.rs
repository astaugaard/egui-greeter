use anyhow::Result;
use clap::Parser;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub user: Option<String>,

    #[arg(short = 'n', long)]
    pub session_name: Option<String>,

    #[arg(short = 'C', long)]
    pub session_command: Option<String>,

    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub default_session_name: String,
    pub default_session_command: String,
    pub user: String,
}

impl Settings {
    pub fn from_args(args: Args) -> Result<Settings> {
        let config_file: OnceCell<Settings> = OnceCell::new();

        let config_file_fn = || -> Result<Settings> {
            let config = match &args.config {
                Some(config) => config.clone(),
                None => "/etc/greetd/egui-greeter.json".into(),
            };

            let settings = fs::read_to_string(config)?;

            let settings: Settings = serde_json::from_str(&settings)?;

            Ok(settings)
        };

        let default_session_name = match args.session_name {
            Some(a) => a,
            None => config_file
                .get_or_try_init(config_file_fn)?
                .default_session_name
                .clone(),
        };

        let default_session_command = match args.session_command {
            Some(a) => a,
            None => config_file
                .get_or_try_init(config_file_fn)?
                .default_session_command
                .clone(),
        };

        let user = match args.user {
            Some(a) => a,
            None => config_file.get_or_try_init(config_file_fn)?.user.clone(),
        };

        Ok(Settings {
            default_session_name,
            default_session_command,
            user,
        })
    }
}
