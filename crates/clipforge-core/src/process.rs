use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(crate) fn tool_command(name: &str) -> Command {
    let mut command = Command::new(tool_path(name));
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

fn tool_path(name: &str) -> PathBuf {
    let filename = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    let bundled = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(&filename)));
    match bundled {
        Some(path) if path.is_file() => path,
        _ => PathBuf::from(name),
    }
}
