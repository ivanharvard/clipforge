use std::path::PathBuf;

use clipforge_core::Project;
use serde::{Deserialize, Serialize};

use crate::settings::config_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoverySnapshot {
    pub projects: Vec<Project>,
    pub active_index: usize,
}

pub fn load() -> Option<RecoverySnapshot> {
    [snapshot_path()?, backup_path()?]
        .into_iter()
        .find_map(|path| {
            let contents = std::fs::read_to_string(path).ok()?;
            serde_json::from_str(&contents).ok()
        })
}

pub fn save(snapshot: &RecoverySnapshot) -> anyhow::Result<()> {
    let path = snapshot_path().ok_or_else(|| anyhow::anyhow!("config directory unavailable"))?;
    let backup = backup_path().ok_or_else(|| anyhow::anyhow!("config directory unavailable"))?;
    let temporary =
        temporary_path().ok_or_else(|| anyhow::anyhow!("config directory unavailable"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&temporary, serde_json::to_vec(snapshot)?)?;
    if path.exists() {
        let _ = std::fs::copy(&path, &backup);
        std::fs::remove_file(&path)?;
    }
    std::fs::rename(temporary, path)?;
    Ok(())
}

pub fn clear() {
    for path in [snapshot_path(), backup_path(), temporary_path()]
        .into_iter()
        .flatten()
    {
        let _ = std::fs::remove_file(path);
    }
}

fn snapshot_path() -> Option<PathBuf> {
    config_dir().map(|path| path.join("recovery.json"))
}

fn backup_path() -> Option<PathBuf> {
    config_dir().map(|path| path.join("recovery.json.bak"))
}

fn temporary_path() -> Option<PathBuf> {
    config_dir().map(|path| path.join("recovery.json.tmp"))
}
