//! Saved adventure progress: how far you have got, so Continue can start there.
//! One number in `$XDG_STATE_HOME/uxo/progress` (or `~/.local/state/uxo/progress`).
//! Progress saved by 0.6 under `$XDG_DATA_HOME` is still read.

use std::path::PathBuf;

/// An environment folder, with an empty value counted as unset.
fn env_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from)
}

fn home() -> Option<PathBuf> {
    env_dir("HOME")
}

fn path() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    let base = env_dir("XDG_STATE_HOME").or_else(|| home().map(|h| h.join(".local/state")))?;
    Some(base.join("uxo/progress"))
}

/// Where 0.6 kept it.
fn old_path() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    let base = env_dir("XDG_DATA_HOME").or_else(|| home().map(|h| h.join(".local/share")))?;
    Some(base.join("uxo/progress"))
}

fn read(p: Option<PathBuf>) -> Option<usize> {
    p.and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| s.trim().parse().ok())
}

/// The furthest stage reached (0 = none cleared yet).
pub fn furthest() -> usize {
    read(path()).or_else(|| read(old_path())).unwrap_or(0)
}

/// Record that stage `n` (1-based count of stages cleared) has been reached.
pub fn reached(n: usize) {
    if n <= furthest() {
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
