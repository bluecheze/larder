use std::path::Path;

use larder_core::scan::is_ignored;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

/// Keep this alive for as long as you want events. Dropping it stops the
/// OS watcher.
pub struct Watch {
    _watcher: RecommendedWatcher,
    pub changes: UnboundedReceiver<()>,
}

/// Fires whenever something under `root` changes. It doesn't say what
/// changed on purpose: the sync pass rescans anyway, and the OS event
/// details differ a lot between macOS, Windows and Linux.
pub fn watch(root: &Path) -> notify::Result<Watch> {
    let (tx, changes) = unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        let Ok(event) = event else { return };
        // Our own temp downloads would otherwise trigger a sync of their own.
        let relevant = event.paths.iter().any(|p| {
            !p.components().any(|c| c.as_os_str().to_str().is_some_and(is_ignored))
        });
        if relevant {
            let _ = tx.send(());
        }
    })?;
    watcher.watch(root, RecursiveMode::Recursive)?;
    Ok(Watch { _watcher: watcher, changes })
}
