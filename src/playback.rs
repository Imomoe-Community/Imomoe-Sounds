// Copyright (C) 2026 Mai Ooizumi
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{
    playlist::{Audience, Config, Order, Playlist, clip_available},
    s2sdk, with_state,
};
use plugify::{Arr, Str, Vec3};
use rand::{SeedableRng, rngs::StdRng};
use s2sdk::CSTeam;
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

#[derive(Clone, Copy)]
pub(crate) enum Target {
    Event(Option<i32>),
    Client(i32),
    Team(i32),
    Spatial {
        source: i32,
        distance_cap: f32,
        tactical: bool,
    },
}

impl Target {
    fn tactical(self) -> bool {
        matches!(self, Self::Spatial { tactical: true, .. })
    }

    fn audience(self, audience: Audience) -> Option<i32> {
        match self {
            Self::Event(actor) if audience == Audience::Actor => Some(actor.unwrap_or(-1)),
            Self::Client(slot) => Some(slot),
            _ => None,
        }
    }

    fn cursor_scope(self) -> CursorScope {
        match self {
            Self::Event(Some(actor)) | Self::Client(actor) => CursorScope::Actor(actor),
            Self::Spatial { source, .. } => CursorScope::Entity(source),
            Self::Event(None) | Self::Team(_) => CursorScope::Global,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum CursorScope {
    Global,
    Actor(i32),
    Entity(i32),
}

#[derive(Clone, Copy)]
enum Listener {
    Player(u64),
    Cstv,
}

#[derive(Default)]
struct Tracks {
    entity: HashMap<(i32, String), String>,
    world: BTreeSet<String>,
}

pub(crate) struct Playback {
    pub config: Arc<Config>,
    pub mounted: BTreeSet<String>,
    next: HashMap<(CursorScope, String), usize>,
    actor_death_ticks: HashMap<i32, i32>,
    rng: StdRng,
    tracks: Tracks,
}

impl Playback {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: Arc::new(config),
            mounted: BTreeSet::new(),
            next: HashMap::new(),
            actor_death_ticks: HashMap::new(),
            rng: StdRng::try_from_rng(&mut rand::rngs::SysRng)?,
            tracks: Tracks::default(),
        })
    }

    pub fn tactical_ready(&self, name: &str) -> bool {
        name.starts_with("funrounds.tactical.")
            && self.config.playlists.get(name).is_some_and(|list| {
                list.clips
                    .iter()
                    .any(|clip| clip_available(clip, self.config.workshop.as_ref(), &self.mounted))
            })
    }

    pub fn reset_map(&mut self) {
        self.next.clear();
        self.actor_death_ticks.clear();
        self.mounted.clear();
        self.tracks = Tracks::default();
    }

    pub fn forget_actor(&mut self, slot: i32) {
        self.actor_death_ticks.remove(&slot);
        if !self.next.is_empty() {
            self.next
                .retain(|(scope, _), _| *scope != CursorScope::Actor(slot));
        }
    }

    pub fn actor_died(&mut self, slot: i32, tick: i32) {
        self.actor_death_ticks.insert(slot, tick);
    }

    pub fn finish_deaths_before(&mut self, tick: i32) {
        if self.actor_death_ticks.is_empty() {
            return;
        }
        self.next.retain(|(scope, _), _| {
            let CursorScope::Actor(slot) = scope else {
                return true;
            };
            self.actor_death_ticks
                .get(slot)
                .is_none_or(|death| *death >= tick)
        });
        self.actor_death_ticks.retain(|_, death| *death >= tick);
    }

    pub fn forget_entity(&mut self, handle: i32) {
        if !self.tracks.entity.is_empty() {
            self.tracks
                .entity
                .retain(|(entity, _), _| *entity != handle);
        }
        if !self.next.is_empty() {
            self.next
                .retain(|(scope, _), _| *scope != CursorScope::Entity(handle));
        }
    }

    pub fn take_interruptible(&mut self) -> BTreeSet<String> {
        std::mem::take(&mut self.tracks.world)
    }

    fn take_spatial(&mut self, name: &str, source: i32) -> Option<String> {
        self.tracks.entity.remove(&(source, name.into()))
    }

    fn prepare_event(
        &mut self,
        name: &str,
        list: &Playlist,
        has_recipients: bool,
        source: Option<i32>,
        scope: CursorScope,
    ) -> (Option<String>, Option<String>) {
        let previous = source.and_then(|source| self.take_spatial(name, source));
        let effective_scope = match list.order {
            Order::ActorSequential => scope,
            Order::Sequential | Order::Random => CursorScope::Global,
        };
        let event = has_recipients.then(|| {
            let mut dummy = 0;
            let next = if list.order == Order::Random {
                &mut dummy
            } else {
                self.next.entry((effective_scope, name.into())).or_default()
            };
            list.pick(
                self.config.workshop.as_ref(),
                &self.mounted,
                next,
                &mut self.rng,
            )
            .map(str::to_owned)
        });
        (event.flatten(), previous)
    }

    pub fn stop_all(&mut self) {
        stop_world(self.take_interruptible());
        for ((source, _), event) in self.tracks.entity.drain() {
            stop_if_valid(source, &event);
        }
    }
}

fn exists(path: &str) -> bool {
    // ReadFileVPK carries compiled binary data in a Plugify string.
    !s2sdk::ReadFileVPK(&Str::from_str(&format!("{path}_c")), &Str::from_str("GAME")).is_empty()
}

pub(crate) fn precache(config: &Config) -> BTreeSet<String> {
    let mut ready = BTreeSet::new();
    let Some(pack) = &config.workshop else {
        return ready;
    };
    if !s2sdk::GetServerAddons().as_slice().contains(&pack.id) {
        plugify::warning!(
            "imomoe_sounds: add Workshop item {} to s2_extra_addons; custom clips disabled",
            pack.id
        );
        return ready;
    }
    if !exists(&pack.events) {
        plugify::warning!(
            "imomoe_sounds: {}_c unavailable; custom clips disabled",
            pack.events
        );
        return ready;
    }
    s2sdk::Precache(&Str::from_str(&pack.events));
    // A sound-event file may refer to assets shared by several cues.
    let mut files = HashMap::new();
    for (event, path) in &pack.resources {
        let exists = *files.entry(path).or_insert_with(|| {
            if exists(path) {
                s2sdk::Precache(&Str::from_str(path));
                true
            } else {
                plugify::warning!(
                    "imomoe_sounds: {}_c unavailable; affected clips disabled",
                    path
                );
                false
            }
        });
        if exists {
            ready.insert(event.clone());
        }
    }
    ready
}

fn world() -> Option<i32> {
    let handle = s2sdk::EntIndexToEntHandle(0);
    (handle != -1).then_some(handle)
}

fn stop_if_valid(source: i32, event: &str) -> bool {
    if !s2sdk::IsValidEntHandle(source) {
        return false;
    }
    s2sdk::StopSound(source, &Str::from_str(event));
    true
}

pub(crate) fn stop_world(events: BTreeSet<String>) {
    if !events.is_empty()
        && let Some(source) = world()
    {
        for event in events {
            s2sdk::StopSound(source, &Str::from_str(&event));
        }
    }
}

pub(crate) fn stop_spatial(name: &str, source: i32) -> bool {
    let event = with_state(|state| state.playback.take_spatial(name, source)).flatten();
    event.is_some_and(|event| stop_if_valid(source, &event))
}

fn listener(slot: i32, team: Option<CSTeam>, include_cstv: bool) -> Option<Listener> {
    if !s2sdk::IsClientInGame(slot) {
        return None;
    }
    if team.is_some_and(|team| s2sdk::GetClientTeam(slot) != team) {
        return None;
    }
    if s2sdk::IsFakeClient(slot) {
        return (include_cstv && s2sdk::IsClientSourceTV(slot)).then_some(Listener::Cstv);
    }
    Some(Listener::Player(s2sdk::GetClientSteamID64(slot)))
}

fn receives(listener: Listener, tactical: bool, enabled: impl FnOnce(u64) -> bool) -> bool {
    match listener {
        Listener::Player(steam) if !tactical => enabled(steam),
        _ => true,
    }
}

fn near(source: Vec3, target: Vec3, distance: f32) -> bool {
    let delta = source - target;
    let x = f64::from(delta.x);
    let y = f64::from(delta.y);
    let z = f64::from(delta.z);
    let distance = f64::from(distance);
    x * x + y * y + z * z <= distance * distance
}

fn cap(requested: f32, configured: Option<f32>) -> Option<f32> {
    match (requested, configured) {
        (0.0, distance) => distance,
        (requested, Some(distance)) => Some(requested.min(distance)),
        (requested, None) => Some(requested),
    }
}

fn finish_prepared_event(
    (event, previous): (Option<String>, Option<String>),
    source: Option<i32>,
    stop: impl FnOnce(i32, &str),
) -> Option<String> {
    if let (Some(source), Some(previous)) = (source, previous) {
        stop(source, &previous);
    }
    event
}

pub(crate) fn dispatch<'a>(names: impl IntoIterator<Item = &'a str>, target: Target) -> bool {
    dispatch_selected(names, target).is_some()
}

pub(crate) fn dispatch_selected<'a>(
    names: impl IntoIterator<Item = &'a str>,
    target: Target,
) -> Option<String> {
    // Check lifecycle/thread before touching any SDK binding.
    let (config, observer_pawn_offset, life_state_offset, observer_schema) = with_state(|state| {
        state.live.then(|| {
            (
                state.playback.config.clone(),
                state.observer_pawn_offset,
                state.life_state_offset,
                state.observer_schema,
            )
        })
    })
    .flatten()?;
    let team = match target {
        Target::Team(2) => Some(CSTeam::T),
        Target::Team(3) => Some(CSTeam::CT),
        Target::Team(value) if value != 0 => return None,
        _ => None,
    };
    let spatial = if let Target::Spatial {
        source,
        distance_cap,
        ..
    } = target
    {
        if !distance_cap.is_finite() || distance_cap < 0.0 || !s2sdk::IsValidEntHandle(source) {
            return None;
        }
        Some((source, distance_cap))
    } else {
        None
    };
    let max = s2sdk::GetMaxClients();
    for name in names {
        let Some(list) = config.playlists.get(name) else {
            continue;
        };
        let tactical = name.starts_with("funrounds.tactical.");
        if tactical != target.tactical() || spatial.is_some() != (tactical || list.spatial) {
            continue;
        }

        // An explicitly empty playlist mutes this event, including its lower-priority cues.
        if list.clips.is_empty() {
            return None;
        }

        // Select the highest-priority configured, available playlist before applying
        // per-listener preferences. A muted actor must not change the cue everyone else hears.
        if !with_state(|state| {
            list.clips
                .iter()
                .any(|clip| clip_available(clip, config.workshop.as_ref(), &state.playback.mounted))
        })
        .unwrap_or(false)
        {
            continue;
        }

        let audience = target.audience(if name == "first_blood" {
            Audience::All
        } else {
            list.audience
        });
        let slots = match audience {
            Some(slot) if (0..max).contains(&slot) => slot..slot + 1,
            Some(_) => return None,
            None => 0..max,
        };
        let origin = spatial.and_then(|(entity, requested)| {
            cap(requested, list.distance)
                .map(|distance| (s2sdk::GetEntityAbsOrigin(entity), distance))
        });
        let listeners: Vec<_> = slots
            .filter_map(|slot| {
                let listener = listener(slot, team, audience.is_none() && team.is_none())?;
                if let Some((source, distance)) = origin
                    && !matches!(listener, Listener::Cstv)
                    && crate::listener_origin(
                        slot,
                        observer_pawn_offset,
                        life_state_offset,
                        observer_schema,
                    )
                    .is_none_or(|position| !near(source, position, distance))
                {
                    return None;
                }
                Some((slot, listener))
            })
            .collect();
        let recipients = with_state(|state| {
            let recipients: Vec<_> = listeners
                .into_iter()
                .filter_map(|(slot, listener)| {
                    receives(listener, tactical, |id| state.preferences.enabled(id)).then_some(slot)
                })
                .collect();
            recipients
        })
        .unwrap_or_default();
        let prepared = with_state(|state| {
            state.playback.prepare_event(
                name,
                list,
                !recipients.is_empty(),
                spatial.map(|(source, _)| source),
                target.cursor_scope(),
            )
        })?;
        let event = finish_prepared_event(
            prepared,
            spatial.map(|(source, _)| source),
            |source, event| {
                stop_if_valid(source, event);
            },
        )?;

        let source = match spatial {
            Some((source, _)) => source,
            None => world()?,
        };
        s2sdk::EmitSoundToClient(
            source,
            &Arr::from(recipients),
            &Str::from_str(&event),
            1.0,
            100.0,
        );
        with_state(|state| {
            if spatial.is_some() {
                state
                    .playback
                    .tracks
                    .entity
                    .insert((source, name.into()), event.clone());
            } else if list.interruptible {
                state.playback.tracks.world.insert(event.clone());
            }
        });
        return Some(event);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_inclusive_uses_z_and_does_not_extend_configured_range() {
        let source = Vec3::new(0., 0., 0.);
        assert!(near(source, Vec3::new(3., 0., 4.), 5.));
        assert!(!near(source, Vec3::new(3., 0., 4.1), 5.));
        assert_eq!(cap(0., Some(1000.)), Some(1000.));
        assert_eq!(cap(600., Some(1000.)), Some(600.));
        assert_eq!(cap(1200., Some(1000.)), Some(1000.));
        assert_eq!(cap(0., None), None);
    }

    #[test]
    fn tactical_readiness_requires_an_available_clip() {
        let config = Config::parse(
            r#"{
          "workshop": {"id": 10, "events": "soundevents/p.vsndevts",
                       "resources": {"Addon.Breath": "sounds/breath.vsnd"}},
          "funrounds": {"tactical": {
              "breath": {"spatial": true, "clips": ["Addon.Breath"]},
              "empty": {"spatial": true, "clips": []}
          }}
        }"#,
        )
        .unwrap();
        let mut playback = Playback::new(config).unwrap();
        assert!(!playback.tactical_ready("funrounds.tactical.breath"));
        playback.mounted.insert("Addon.Breath".into());
        assert!(playback.tactical_ready("funrounds.tactical.breath"));
        assert!(!playback.tactical_ready("funrounds.tactical.empty"));
        assert!(!playback.tactical_ready("missing"));
        playback.reset_map();
        assert!(!playback.tactical_ready("funrounds.tactical.breath"));
    }

    #[test]
    fn player_mute_only_applies_to_non_tactical_human_listeners() {
        assert!(receives(Listener::Player(42), false, |_| true));
        assert!(!receives(Listener::Player(42), false, |_| false));
        assert!(receives(Listener::Player(42), true, |_| false));
        assert!(receives(Listener::Cstv, false, |_| false));
    }

    #[test]
    fn empty_spatial_audience_releases_previous_track_without_advancing_playlist() {
        let config = Config::parse(
            r#"{"funrounds":{"tactical":{"breath":{"spatial":true,"clips":["A","B"]}}}}"#,
        )
        .unwrap();
        let mut playback = Playback::new(config).unwrap();
        let config = playback.config.clone();
        let name = "funrounds.tactical.breath";
        let list = &config.playlists[name];
        playback
            .tracks
            .entity
            .insert((12, name.into()), "old".into());

        let prepared = playback.prepare_event(name, list, false, Some(12), CursorScope::Entity(12));
        let mut stopped = None;
        let event = finish_prepared_event(prepared, Some(12), |source, event| {
            stopped = Some((source, event.to_owned()));
        });
        assert_eq!(event, None);
        assert_eq!(stopped, Some((12, "old".into())));
        assert!(!playback.tracks.entity.contains_key(&(12, name.into())));
        assert!(
            !playback
                .next
                .contains_key(&(CursorScope::Entity(12), name.into()))
        );

        let (event, previous) =
            playback.prepare_event(name, list, true, Some(12), CursorScope::Entity(12));
        assert_eq!(event.as_deref(), Some("A"));
        assert_eq!(previous, None);
        let (event, _) =
            playback.prepare_event(name, list, true, Some(12), CursorScope::Entity(12));
        assert_eq!(event.as_deref(), Some("B"));
    }

    #[test]
    fn actor_sequential_tracks_independent_cursors_and_cleans_up_per_scope() {
        let config = Config::parse(
            r#"{
                "announcements": {"kill": {"order": "actor_sequential", "clips": ["K1", "K2", "K3"]}},
                "funrounds": {"tactical": {"breath": {"order": "actor_sequential", "spatial": true, "clips": ["B1", "B2"]}}}
            }"#,
        )
        .unwrap();
        let mut playback = Playback::new(config).unwrap();
        let config = playback.config.clone();
        let kill = &config.playlists["kill"];
        let breath = &config.playlists["funrounds.tactical.breath"];

        assert_eq!(
            playback
                .prepare_event("kill", kill, true, None, CursorScope::Actor(1))
                .0
                .as_deref(),
            Some("K1")
        );
        assert_eq!(
            playback
                .prepare_event("kill", kill, true, None, CursorScope::Actor(2))
                .0
                .as_deref(),
            Some("K1")
        );
        assert_eq!(
            playback
                .prepare_event("kill", kill, true, None, CursorScope::Actor(1))
                .0
                .as_deref(),
            Some("K2")
        );
        playback.forget_actor(1);
        assert_eq!(
            playback
                .prepare_event("kill", kill, true, None, CursorScope::Actor(1))
                .0
                .as_deref(),
            Some("K1")
        );
        assert_eq!(
            playback
                .prepare_event("kill", kill, true, None, CursorScope::Actor(2))
                .0
                .as_deref(),
            Some("K2")
        );

        assert_eq!(
            playback
                .prepare_event(
                    "funrounds.tactical.breath",
                    breath,
                    true,
                    Some(100),
                    CursorScope::Entity(100)
                )
                .0
                .as_deref(),
            Some("B1")
        );
        assert_eq!(
            playback
                .prepare_event(
                    "funrounds.tactical.breath",
                    breath,
                    true,
                    Some(200),
                    CursorScope::Entity(200)
                )
                .0
                .as_deref(),
            Some("B1")
        );
        playback.actor_died(1, 10);
        playback.finish_deaths_before(11);
        playback.forget_entity(100);
        assert_eq!(
            playback
                .prepare_event(
                    "funrounds.tactical.breath",
                    breath,
                    true,
                    Some(100),
                    CursorScope::Entity(100)
                )
                .0
                .as_deref(),
            Some("B1")
        );
        assert_eq!(
            playback
                .prepare_event(
                    "funrounds.tactical.breath",
                    breath,
                    true,
                    Some(200),
                    CursorScope::Entity(200)
                )
                .0
                .as_deref(),
            Some("B2")
        );
    }
}
