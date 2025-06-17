use freedesktop_desktop_entry::DesktopEntry;
use std::{env, path::PathBuf};

pub fn get_sessions(
    default: String,
    default_command: String,
) -> Vec<(Option<PathBuf>, (String, String))> {
    // copied and modified from tuigreet
    let xdg_data_dirs: Vec<PathBuf> = {
        let value = env::var("XDG_DATA_DIRS").unwrap_or("/usr/local/share:/usr/share".to_string());
        env::split_paths(&value)
            .filter(|p| p.is_absolute())
            .collect()
    };

    let default_session_paths: Vec<PathBuf> = xdg_data_dirs
        .iter()
        .map(|p| (p.join("wayland-sessions")))
        .chain(xdg_data_dirs.iter().map(|p| (p.join("xsessions"))))
        .collect();

    let default_sessions: Vec<(Option<PathBuf>, (String, String))> =
        [(None, (default, default_command))]
            .into_iter()
            .chain(
                default_session_paths
                    .iter()
                    .flat_map(|pathbuf| match pathbuf.read_dir() {
                        Ok(path) => {
                            let entries: Vec<(Option<PathBuf>, (String, String))> = path
                                .flatten()
                                .flat_map(|child| {
                                    let path = child.path();
                                    let n: Option<&[&str]> = None;
                                    let entry = DesktopEntry::from_path(&path, n).ok()?;

                                    let name = entry.name::<&str>(&[])?;
                                    let command = entry.parse_exec().ok()?;

                                    let mut ncommand = String::new();

                                    for n in command {
                                        ncommand.push_str(n.as_str());
                                        ncommand.push(' ');
                                    }

                                    Some((Some(path), (name.to_string(), ncommand)))
                                })
                                .collect::<Vec<_>>();
                            entries
                        }
                        Err(_) => Vec::new(),
                    }),
            )
            .collect::<Vec<_>>();

    default_sessions
}
