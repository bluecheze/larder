use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Pinned {
    /// It's in the sidebar now. Never happens on macOS, see `pin` below.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    Done,
    /// The OS won't let us, the UI shows how to drag it in by hand.
    Manual,
}

/// Puts the Larder folder in the file manager's sidebar.
#[cfg(target_os = "windows")]
pub fn pin(folder: &Path) -> Result<Pinned, String> {
    // "pintohome" is the same verb Explorer runs when you right-click a
    // folder and pick "Pin to Quick access". Single quotes get doubled
    // because that's how PowerShell escapes them inside '...'.
    let path = folder.to_string_lossy().replace('\'', "''");
    let script = format!("(New-Object -ComObject Shell.Application).Namespace('{path}').Self.InvokeVerb('pintohome')");
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() { Ok(Pinned::Done) } else { Err("Explorer didn't accept the pin".into()) }
}

/// GTK's bookmarks file is what Nautilus, Nemo, Thunar and the GTK file
/// pickers all read. KDE keeps its own list and isn't covered yet.
#[cfg(target_os = "linux")]
pub fn pin(folder: &Path) -> Result<Pinned, String> {
    let home = std::env::var_os("HOME").ok_or("no home folder")?;
    let file = Path::new(&home).join(".config/gtk-3.0/bookmarks");
    let line = format!("file://{} Larder", encode(&folder.to_string_lossy()));
    let existing = std::fs::read_to_string(&file).unwrap_or_default();
    if existing.lines().any(|l| l.split(' ').next() == line.split(' ').next()) {
        return Ok(Pinned::Done);
    }
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&line);
    text.push('\n');
    std::fs::write(&file, text).map_err(|e| e.to_string())?;
    Ok(Pinned::Done)
}

#[cfg(target_os = "linux")]
fn encode(path: &str) -> String {
    path.bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// macOS used to have an API for Finder favorites. It's been dead for a
/// few releases (calling it crashes on macOS 27) and sfltool lost its add
/// command, so the honest option is opening the folder in Finder and
/// letting the user drag it into the sidebar once.
#[cfg(target_os = "macos")]
pub fn pin(folder: &Path) -> Result<Pinned, String> {
    std::process::Command::new("open").arg(folder).status().map_err(|e| e.to_string())?;
    Ok(Pinned::Manual)
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn pin(_folder: &Path) -> Result<Pinned, String> {
    Ok(Pinned::Manual)
}
