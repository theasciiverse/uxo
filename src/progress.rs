//! Saved adventure progress: how far you have got, so Continue can start there.
//! One number in `$XDG_DATA_HOME/uxo/progress` (or `~/.local/share/uxo/progress`).

use std::path::PathBuf;

fn path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("uxo/progress"))
}

/// The furthest stage reached (0 = none cleared yet).
pub fn furthest() -> usize {
    path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| s.trim().parse().ok()).unwrap_or(0)
}

/// Record that stage `n` (1-based count of stages cleared) has been reached.
pub fn reached(n: usize) {
    if cfg!(test) || n <= furthest() {
        return;
    }
    if let Some(p) = path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&p, n.to_string()) {
            log::warn!("could not save progress to {}: {e}", p.display());
        }
    }
}
