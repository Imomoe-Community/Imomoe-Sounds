// Copyright (C) 2026 Mai Ooizumi
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs, io,
    path::PathBuf,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredPreference {
    enabled: Option<bool>,
    // R3 used zero as a second way to mute; preserve that choice on upgrade.
    volume: Option<u8>,
}

#[derive(Serialize)]
struct SavedPreference {
    enabled: bool,
}

pub(crate) struct Preferences {
    path: PathBuf,
    enabled_by_default: bool,
    overrides: BTreeSet<u64>,
    pending: BTreeMap<u64, bool>,
    dirty: bool,
}

impl Preferences {
    pub fn new(path: PathBuf, enabled_by_default: bool) -> Self {
        Self {
            path,
            enabled_by_default,
            overrides: BTreeSet::new(),
            pending: BTreeMap::new(),
            dirty: false,
        }
    }

    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let result = self.read();
        let players = result?;
        self.dirty = players.values().any(|value| value.volume.is_some());
        self.overrides = players
            .into_iter()
            .filter_map(|(id, value)| {
                let enabled =
                    value.enabled.unwrap_or(self.enabled_by_default) && value.volume != Some(0);
                (enabled != self.enabled_by_default).then_some(id)
            })
            .collect();
        self.pending.clear();
        Ok(())
    }

    fn read(&self) -> Result<BTreeMap<u64, StoredPreference>, Box<dyn Error>> {
        let data = match fs::read(&self.path) {
            Ok(data) => data,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(e) => return Err(e.into()),
        };
        let players: BTreeMap<u64, StoredPreference> = serde_json::from_slice(&data)?;
        if players.keys().any(|&id| id == 0) {
            return Err("expected non-zero SteamID64 keys".into());
        }
        Ok(players)
    }

    pub fn enabled(&self, steam: u64) -> bool {
        self.enabled_by_default ^ self.overrides.contains(&steam)
    }

    pub fn toggle(&mut self, steam: u64) -> (bool, Result<(), Box<dyn Error>>) {
        if !self.overrides.remove(&steam) {
            self.overrides.insert(steam);
        }
        self.pending.insert(steam, self.enabled(steam));
        self.dirty = true;
        let saved = self.flush();
        (self.enabled(steam), saved)
    }

    pub fn flush(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.dirty {
            return Ok(());
        }
        // Merge with the latest file so external edits for untouched players survive.
        // Reads that still fail leave the source file intact and the pending toggles retryable.
        let mut current = self.read()?;
        for (id, enabled) in &self.pending {
            current.insert(
                *id,
                StoredPreference {
                    enabled: Some(*enabled),
                    volume: None,
                },
            );
        }
        let players: BTreeMap<_, _> = current
            .into_iter()
            .filter_map(|(id, value)| {
                let enabled =
                    value.enabled.unwrap_or(self.enabled_by_default) && value.volume != Some(0);
                (enabled != self.enabled_by_default).then_some((id, SavedPreference { enabled }))
            })
            .collect();
        let tmp = tempfile::NamedTempFile::new_in(self.path.parent().expect("config directory"))?;
        serde_json::to_writer_pretty(tmp.as_file(), &players)?;
        tmp.as_file().sync_all()?;
        tmp.persist(&self.path)?;
        self.overrides = players.keys().copied().collect();
        self.pending.clear();
        self.dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_survives_immediate_reload_and_return_to_default_removes_the_entry() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("player_preferences.json");
        let mut prefs = Preferences::new(path.clone(), true);
        prefs.load().unwrap();
        assert!(prefs.enabled(1002));
        assert!(!path.exists());
        let (enabled, saved) = prefs.toggle(1001);
        assert!(!enabled);
        saved.unwrap();

        let mut restored = Preferences::new(path.clone(), true);
        restored.load().unwrap();
        assert!(!restored.enabled(1001));
        assert!(restored.enabled(1002));
        let (enabled, saved) = restored.toggle(1001);
        assert!(enabled);
        saved.unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), "{}");
    }

    #[test]
    fn r3_migration_preserves_mute_and_removes_old_volume_fields() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("player_preferences.json");
        fs::write(
            &path,
            r#"{"1001":{"enabled":false,"volume":30},"1002":{"volume":30},"1003":{"volume":0}}"#,
        )
        .unwrap();
        let mut prefs = Preferences::new(path.clone(), true);
        prefs.load().unwrap();
        assert!(!prefs.enabled(1001));
        assert!(prefs.enabled(1002));
        assert!(!prefs.enabled(1003));
        prefs.flush().unwrap();
        let stored: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(
            stored,
            serde_json::json!({"1001":{"enabled":false},"1003":{"enabled":false}})
        );
    }

    #[test]
    fn malformed_file_is_preserved_and_toggle_reports_unsaved_session_state() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("player_preferences.json");
        fs::write(&path, "not valid JSON").unwrap();
        let mut prefs = Preferences::new(path.clone(), true);
        assert!(prefs.load().is_err());
        let (enabled, saved) = prefs.toggle(1001);
        assert!(!enabled);
        assert!(saved.is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "not valid JSON");
    }

    #[test]
    fn flush_merges_external_changes_for_untouched_players() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("player_preferences.json");
        let mut prefs = Preferences::new(path.clone(), true);
        prefs.load().unwrap();
        fs::write(&path, r#"{"1002":{"enabled":false}}"#).unwrap();
        assert!(!prefs.toggle(1001).0);
        assert!(!prefs.enabled(1002));
        let stored: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(
            stored,
            serde_json::json!({"1001":{"enabled":false},"1002":{"enabled":false}})
        );
    }

    #[test]
    fn pending_toggle_can_be_saved_after_malformed_file_is_repaired() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("player_preferences.json");
        fs::write(&path, "not valid JSON").unwrap();
        let mut prefs = Preferences::new(path.clone(), true);
        assert!(prefs.load().is_err());
        assert!(prefs.toggle(1001).1.is_err());
        fs::write(&path, "{}").unwrap();
        prefs.flush().unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&fs::read(path).unwrap()).unwrap(),
            serde_json::json!({"1001":{"enabled":false}})
        );
    }
}
