// Copyright (C) 2026 Mai Ooizumi
// SPDX-License-Identifier: AGPL-3.0-or-later

use jsonc_parser::{ParseOptions, parse_to_serde_value};
use rand::RngExt;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    #[serde(rename = "$schema")]
    _schema: Option<String>,
    #[serde(default = "yes")]
    enabled_by_default: bool,
    #[serde(default = "default_shutdown")]
    shutdown_min_streak: usize,
    #[serde(default = "default_combo_window")]
    combo_window_seconds: f64,
    chat_prefix_marker: Option<String>,
    workshop: Option<Workshop>,
    #[serde(default)]
    announcements: Announcements,
    #[serde(default)]
    weapons: BTreeMap<String, Playlist>,
    #[serde(default)]
    funrounds: FunRounds,
}

pub(crate) struct Config {
    pub enabled_by_default: bool,
    pub shutdown_min_streak: usize,
    pub combo_window_seconds: f64,
    pub chat_prefix_marker: Option<String>,
    pub workshop: Option<Workshop>,
    pub playlists: BTreeMap<String, Playlist>,
}

#[derive(Default, Deserialize)]
struct Announcements {
    #[serde(default)]
    streak: BTreeMap<String, Playlist>,
    #[serde(default)]
    combo: BTreeMap<String, Playlist>,
    #[serde(default)]
    headshot_streak: BTreeMap<String, Playlist>,
    #[serde(flatten)]
    cues: BTreeMap<String, Playlist>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FunRounds {
    #[serde(default)]
    flavor: BTreeMap<String, Playlist>,
    #[serde(default)]
    tactical: BTreeMap<String, Playlist>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Workshop {
    pub id: u64,
    pub events: String,
    // Declare the asset once, even when multiple playlists reuse its event.
    pub resources: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Order {
    #[default]
    Sequential,
    ActorSequential,
    Random,
}

#[derive(Clone, Copy, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Audience {
    #[default]
    All,
    Actor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Playlist {
    #[serde(default)]
    pub order: Order,
    #[serde(default)]
    pub audience: Audience,
    #[serde(default)]
    pub interruptible: bool,
    #[serde(default)]
    pub spatial: bool,
    pub distance: Option<f32>,
    pub clips: Vec<String>,
}

fn yes() -> bool {
    true
}
fn default_shutdown() -> usize {
    3
}
fn default_combo_window() -> f64 {
    2.0
}

fn valid_event(event: &str) -> bool {
    !event.is_empty() && !event.chars().any(|c| c.is_control() || c.is_whitespace())
}

fn cue_component(name: &str) -> bool {
    valid_event(name) && !name.contains('.')
}

fn resource_path(path: &str, dir: &str, extension: &str) -> bool {
    path.starts_with(dir)
        && path.ends_with(extension)
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/_.-".contains(&b))
}

impl Config {
    pub fn parse(input: &str) -> Result<Self, Box<dyn Error>> {
        let raw: ConfigFile = parse_to_serde_value(
            input.trim_start_matches('\u{feff}'),
            &ParseOptions {
                allow_comments: true,
                allow_trailing_commas: true,
                allow_loose_object_property_names: false,
                allow_missing_commas: false,
                allow_single_quoted_strings: false,
                allow_hexadecimal_numbers: false,
                allow_unary_plus_numbers: false,
                allow_bare_decimal_point_numbers: false,
                allow_non_finite_numbers: false,
                allow_extended_string_escapes: false,
            },
        )?;
        if raw.shutdown_min_streak == 0 {
            return Err("shutdown_min_streak must be positive".into());
        }
        if !raw.combo_window_seconds.is_finite() || raw.combo_window_seconds < 0.0 {
            return Err("combo_window_seconds must be finite and nonnegative".into());
        }
        if raw
            .chat_prefix_marker
            .as_ref()
            .is_some_and(|marker| marker.is_empty() || marker.chars().any(char::is_control))
        {
            return Err(
                "chat_prefix_marker must be nonempty and contain no control characters".into(),
            );
        }
        if let Some(pack) = &raw.workshop
            && (pack.id == 0
                || !resource_path(&pack.events, "soundevents/", ".vsndevts")
                || pack.resources.iter().any(|(event, file)| {
                    !valid_event(event) || !resource_path(file, "sounds/", ".vsnd")
                }))
        {
            return Err("workshop requires a non-zero ID, soundevents/*.vsndevts and valid event → sounds/*.vsnd entries".into());
        }

        let mut playlists = BTreeMap::new();
        for (prefix, lists) in [
            ("", raw.announcements.cues),
            ("weapons.", raw.weapons),
            ("funrounds.flavor.", raw.funrounds.flavor),
            ("funrounds.tactical.", raw.funrounds.tactical),
        ] {
            for (name, list) in lists {
                let cue = format!("{prefix}{name}");
                let round_result = prefix.is_empty()
                    && matches!(
                        name.as_str(),
                        "round_end.ct" | "round_end.t" | "round_end.draw"
                    );
                if (!cue_component(&name) && !round_result)
                    || (prefix.is_empty()
                        && matches!(name.as_str(), "streak" | "combo" | "headshot_streak"))
                {
                    return Err(format!("{cue}: invalid cue component").into());
                }
                if round_result && list.audience != Audience::All {
                    return Err(format!(
                        "{cue}: round results have no actor; audience must be all"
                    )
                    .into());
                }
                let tactical = prefix == "funrounds.tactical.";
                Self::validate_playlist(&cue, &list, tactical)?;
                playlists.insert(cue, list);
            }
        }
        for (threshold_prefix, thresholds) in [
            ("streak", raw.announcements.streak),
            ("combo", raw.announcements.combo),
            ("headshot_streak", raw.announcements.headshot_streak),
        ] {
            for (threshold, list) in thresholds {
                let cue = format!("{threshold_prefix}.{threshold}");
                if threshold.starts_with('0')
                    || !threshold.bytes().all(|b| b.is_ascii_digit())
                    || threshold.parse::<usize>().is_err()
                {
                    return Err(format!("{cue}: threshold must be positive").into());
                }
                Self::validate_playlist(&cue, &list, false)?;
                playlists.insert(cue, list);
            }
        }

        Ok(Self {
            enabled_by_default: raw.enabled_by_default,
            shutdown_min_streak: raw.shutdown_min_streak,
            combo_window_seconds: raw.combo_window_seconds,
            chat_prefix_marker: raw.chat_prefix_marker,
            workshop: raw.workshop,
            playlists,
        })
    }

    fn validate_playlist(
        name: &str,
        list: &Playlist,
        tactical: bool,
    ) -> Result<(), Box<dyn Error>> {
        if list.clips.iter().any(|clip| !valid_event(clip)) {
            return Err(format!("{name}: invalid event").into());
        }
        if matches!(
            name,
            "funrounds.flavor.player_blind" | "funrounds.flavor.regeneration"
        ) && list.spatial
        {
            return Err(format!("{name}: this player cue must be nonspatial").into());
        }
        if tactical && !list.spatial
            || list.spatial && !tactical && !name.starts_with("funrounds.flavor.")
            || list.distance.is_some() && !list.spatial && !tactical
            || (tactical || list.spatial) && (list.interruptible || list.audience != Audience::All)
            || list
                .distance
                .is_some_and(|distance| !distance.is_finite() || distance <= 0.0)
        {
            return Err(format!("{name}: invalid spatial settings or audience").into());
        }
        Ok(())
    }
}

pub(crate) fn clip_available(
    event: &str,
    workshop: Option<&Workshop>,
    mounted: &BTreeSet<String>,
) -> bool {
    workshop.is_none_or(|pack| !pack.resources.contains_key(event) || mounted.contains(event))
}

impl Playlist {
    pub fn pick<'a>(
        &'a self,
        workshop: Option<&Workshop>,
        mounted: &BTreeSet<String>,
        next: &mut usize,
        rng: &mut impl RngExt,
    ) -> Option<&'a str> {
        let eligible = || {
            self.clips
                .iter()
                .filter(|clip| clip_available(clip, workshop, mounted))
        };
        let count = eligible().count();
        if count == 0 {
            return None;
        }
        let index = match self.order {
            Order::Sequential | Order::ActorSequential => {
                let index = *next % count;
                *next = (index + 1) % count;
                index
            }
            Order::Random => rng.random_range(0..count),
        };
        eligible().nth(index).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    const PACK: &str = r#"{
        "workshop": {"id": 1, "events": "soundevents/addon.vsndevts",
                     "resources": {"Addon.B": "sounds/b.vsnd"}},
        "announcements": {"kill": {"clips": ["UI.ArmsRace.Kill1", "Addon.B", "C"]}}
    }"#;

    #[test]
    fn public_selection_wraps_and_skips_missing_assets() {
        let config = Config::parse(PACK).unwrap();
        let mut rng = StdRng::seed_from_u64(1);
        let mut next = 0;
        let mounted = BTreeSet::from(["Addon.B".into()]);
        let playlist = &config.playlists["kill"];
        let mut pick = |ready: &BTreeSet<String>| {
            playlist
                .pick(config.workshop.as_ref(), ready, &mut next, &mut rng)
                .unwrap()
        };
        assert_eq!(pick(&mounted), "UI.ArmsRace.Kill1");
        assert_eq!(pick(&mounted), "Addon.B");
        assert_eq!(pick(&mounted), "C");
        assert_eq!(pick(&mounted), "UI.ArmsRace.Kill1");
        assert_eq!(pick(&BTreeSet::new()), "C");
    }

    #[test]
    fn random_draws_only_eligible_clips() {
        let config =
            Config::parse(&PACK.replace(r#""clips":"#, r#""order": "random", "clips":"#)).unwrap();
        let mut rng = StdRng::seed_from_u64(4);
        let mut next = 0;
        let choices: BTreeSet<_> = (0..32)
            .map(|_| {
                config.playlists["kill"]
                    .pick(
                        config.workshop.as_ref(),
                        &BTreeSet::new(),
                        &mut next,
                        &mut rng,
                    )
                    .unwrap()
            })
            .collect();
        assert_eq!(choices, BTreeSet::from(["UI.ArmsRace.Kill1", "C"]));
        assert_eq!(next, 0);
    }

    #[test]
    fn configured_outcomes_and_markers_are_validated_without_changing_other_cue_names() {
        let config = Config::parse(
            r#"{
            "chat_prefix_marker": "{prefix}",
            "announcements": {
                "round_end": {"clips": ["Default"]},
                "round_end.ct": {"clips": ["CounterTerrorist.Win"]},
                "round_end.t": {"clips": []},
                "round_end.draw": {"clips": ["Addon.Draw"]}
            }
        }"#,
        )
        .unwrap();
        assert_eq!(config.chat_prefix_marker.as_deref(), Some("{prefix}"));
        assert!(config.playlists["round_end.t"].clips.is_empty());
        assert!(Config::parse(r#"{"chat_prefix_marker":""}"#).is_err());
        assert!(Config::parse(r#"{"chat_prefix_marker":"a\nb"}"#).is_err());
        assert!(Config::parse(r#"{"announcements":{"round_end.other":{"clips":[]}}}"#).is_err());
        assert!(
            Config::parse(r#"{"announcements":{"round_end.ct":{"audience":"actor","clips":[]}}}"#)
                .is_err()
        );
    }

    #[test]
    fn targeted_funrounds_cues_reject_spatial_but_other_flavors_remain_configurable() {
        for cue in ["player_blind", "regeneration"] {
            let config = format!(
                r#"{{"funrounds":{{"flavor":{{"{cue}":{{"spatial":true,"clips":["A"]}}}}}}}}"#
            );
            assert!(Config::parse(&config).is_err(), "{cue}");
        }
        assert!(
            Config::parse(r#"{"funrounds":{"flavor":{"other":{"spatial":true,"clips":["A"]}}}}"#)
                .is_ok()
        );
        assert!(Config::parse(r#"{"funrounds":{"flavor":{"player_blind":{"clips":[]},"regeneration":{"spatial":false,"clips":[]}}}}"#).is_ok());
    }

    #[test]
    fn shipped_presets_parse_and_reject_obsolete_options_and_invalid_paths() {
        let native = Config::parse(include_str!(
            "../configs/imomoe_sounds/sounds.example.jsonc"
        ))
        .unwrap();
        let workshop = Config::parse(include_str!(
            "../configs/imomoe_sounds/sounds.workshop.example.jsonc"
        ))
        .unwrap();
        assert!(
            native
                .playlists
                .contains_key("funrounds.tactical.ghost_breath")
        );
        assert!(
            workshop
                .workshop
                .unwrap()
                .resources
                .contains_key("Imomoe.kill_a")
        );
        assert!(Config::parse(&PACK.replace("sounds/b.vsnd", "sounds/../b.vsnd")).is_err());
        assert!(
            Config::parse(&PACK.replace(r#""kill": {"#, r#""kill": {"volume": 0.5, "#)).is_err()
        );
        assert!(
            Config::parse(&PACK.replace(r#""UI.ArmsRace.Kill1""#, r#"{"event":"A","volume":0.5}"#))
                .is_err()
        );
        assert!(
            Config::parse(&PACK.replace(r#""workshop":"#, r#""default_volume":100, "workshop":"#))
                .is_err()
        );
        assert!(Config::parse(&PACK.replace(r#""UI.ArmsRace.Kill1""#, r#""bad name""#)).is_err());
    }

    #[test]
    fn combo_headshot_streak_and_actor_sequential_parse_and_validate() {
        let config = Config::parse(
            r#"{
            "combo_window_seconds": 2.5,
            "announcements": {
                "kill": {"order": "actor_sequential", "clips": ["K1", "K2"]},
                "combo": {"2": {"clips": ["C2"]}},
                "headshot_streak": {"1": {"clips": ["HS1"]}, "2": {"clips": ["HS2"]}},
                "headshot": {"clips": ["HSFallback"]}
            }
        }"#,
        )
        .unwrap();
        assert_eq!(config.combo_window_seconds, 2.5);
        assert!(config.playlists["kill"].order == Order::ActorSequential);
        assert!(config.playlists.contains_key("combo.2"));
        assert!(config.playlists.contains_key("headshot_streak.1"));
        assert!(config.playlists.contains_key("headshot_streak.2"));
        assert!(config.playlists.contains_key("headshot"));
        assert!(Config::parse(r#"{"combo_window_seconds": -0.1}"#).is_err());
        assert!(Config::parse(r#"{"announcements":{"combo":{"0":{"clips":[]}}}}"#).is_err());
        assert!(
            Config::parse(r#"{"announcements":{"headshot_streak":{"0":{"clips":[]}}}}"#).is_err()
        );
    }
}
