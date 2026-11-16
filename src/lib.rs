// Copyright (C) 2026 Mai Ooizumi
// SPDX-License-Identifier: AGPL-3.0-or-later

mod chat_prefix;
mod combat;
mod playback;
mod playlist;
mod preferences;
#[allow(dead_code, non_camel_case_types)]
mod s2sdk;
#[cfg(test)]
mod test_support;

use combat::{Combat, classify};
use playback::{Playback, Target, dispatch};
use playlist::Config;
use plugify::{Arr, Str, register_plugin};
use preferences::Preferences;
use s2sdk::{CSTeam, ConCommandContext, ConVarFlag, EventHookError, HookMode, ResultType};
use std::error::Error;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Mutex;
use std::thread::{self, ThreadId};

const EVENTS: &[&str] = &[
    "round_prestart",
    "round_start",
    "round_freeze_end",
    "round_end",
    "player_connect_full",
    "player_death",
    "round_mvp",
    "player_disconnect",
];
const SOUND_COMMAND: &str = "sound";

static STATE: Mutex<Option<State>> = Mutex::new(None);

struct State {
    thread: ThreadId,
    playback: Playback,
    preferences: Preferences,
    combat: Combat,
    warmup_combat: Combat,
    teammates_are_enemies: u64,
    live: bool,
    hooks: Vec<&'static str>,
    sound_registered: bool,
    precache: bool,
    pending_round_start: Option<i32>,
    special_round_bgm_tick: Option<i32>,
    observer_schema: Option<ObserverSchema>,
    life_state_offset: Option<i32>,
    observer_pawn_offset: Option<i32>,
}

#[derive(Clone, Copy)]
pub(crate) struct ObserverSchema {
    controller_observer_pawn: i32,
    pawn_observer_services: i32,
    observer_mode: i32,
    observer_target: i32,
}

fn with_state<T>(action: impl FnOnce(&mut State) -> T) -> Option<T> {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    let state = guard
        .as_mut()
        .filter(|state| state.thread == thread::current().id())?;
    Some(action(state))
}

fn boundary<T>(name: &str, default: T, action: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(action)) {
        Ok(value) => value,
        Err(_) => {
            plugify::error!("imomoe_sounds: panic in {}", name);
            default
        }
    }
}

fn require_bindings(precache: bool) -> Result<(), Box<dyn Error>> {
    macro_rules! require {
        ($($name:ident),* $(,)?) => {
            $(if unsafe { std::ptr::addr_of!(s2sdk::$name).read() }.is_none() {
                return Err(format!("missing S2SDK binding {}", stringify!($name)).into());
            })*
        };
    }
    require!(
        __s2sdk_IsClientAlive,
        __s2sdk_AddConsoleCommand,
        __s2sdk_RemoveCommand,
        __s2sdk_HasConsoleCommandCallbacks,
        __s2sdk_ReplyToCommand,
        __s2sdk_HookEvent,
        __s2sdk_UnhookEvent,
        __s2sdk_GetEventBool,
        __s2sdk_GetEventString,
        __s2sdk_GetEventPlayerSlot,
        __s2sdk_GetMaxClients,
        __s2sdk_GetClientTeam,
        __s2sdk_IsClientInGame,
        __s2sdk_IsFakeClient,
        __s2sdk_IsClientSourceTV,
        __s2sdk_GetClientSteamID64,
        __s2sdk_GetClientLanguageCode,
        __s2sdk_EntIndexToEntHandle,
        __s2sdk_IsValidEntHandle,
        __s2sdk_GetClientPawn,
        __s2sdk_GetClientAbsOrigin,
        __s2sdk_GetEntityAbsOrigin,
        __s2sdk_EmitSoundToClient,
        __s2sdk_StopSound,
        __s2sdk_GetGameTickCount,
        __s2sdk_FindConVar,
        __s2sdk_GetConVarBool,
        __s2sdk_GetGameRules,
        __s2sdk_GetSchemaOffset,
        __s2sdk_GetEntSchema2,
        __s2sdk_EntHandleToEntPointer,
        __s2sdk_OnMapStart_Register,
        __s2sdk_OnMapStart_Unregister,
        __s2sdk_OnMapEnd_Register,
        __s2sdk_OnMapEnd_Unregister,
        __s2sdk_OnEntityDeleted_Register,
        __s2sdk_OnEntityDeleted_Unregister,
        __s2sdk_OnGameFrame_Register,
        __s2sdk_OnGameFrame_Unregister,
    );
    if precache {
        require!(
            __s2sdk_GetServerAddons,
            __s2sdk_Precache,
            __s2sdk_ReadFileVPK,
            __s2sdk_OnBuildGameSessionManifest_Register,
            __s2sdk_OnBuildGameSessionManifest_Unregister,
        );
    }
    Ok(())
}

fn prepare_state() -> Result<State, Box<dyn Error>> {
    let root = plugify::CONFIGS_DIR
        .get()
        .ok_or("Plugify config directory unavailable")?;
    let dir = Path::new(root).join("imomoe_sounds");
    let path = dir.join("sounds.jsonc");
    let config = std::fs::read_to_string(&path)
        .map_err(Box::<dyn Error>::from)
        .and_then(|text| Config::parse(&text))
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let precache = config.workshop.is_some();
    require_bindings(precache)?;
    let mut preferences = Preferences::new(
        dir.join("player_preferences.json"),
        config.enabled_by_default,
    );
    if let Err(error) = preferences.load() {
        plugify::warning!(
            "imomoe_sounds: cannot read player_preferences.json: {}; using session preferences",
            error
        );
    }
    let state = State {
        thread: thread::current().id(),
        playback: Playback::new(config)?,
        preferences,
        combat: Combat::default(),
        warmup_combat: Combat::default(),
        teammates_are_enemies: s2sdk::FindConVar(&Str::from_str("mp_teammates_are_enemies")),
        live: false,
        hooks: Vec::new(),
        sound_registered: false,
        precache,
        pending_round_start: None,
        special_round_bgm_tick: None,
        observer_schema: None,
        life_state_offset: None,
        observer_pawn_offset: None,
    };
    Ok(state)
}

fn start() -> Result<(), Box<dyn Error>> {
    boundary(
        "start",
        Err("imomoe_sounds: initialization panicked".into()),
        || {
            let state = match prepare_state() {
                Ok(state) => state,
                Err(error) => {
                    plugify::error!("imomoe_sounds: {}; sounds disabled", error);
                    return Ok(());
                }
            };
            let precache = state.precache;
            *STATE.lock().unwrap_or_else(|e| e.into_inner()) = Some(state);
            if unsafe { std::ptr::addr_of!(s2sdk::__s2sdk_GetEventInt).read() }.is_none() {
                plugify::warning!(
                    "imomoe_sounds: GetEventInt unavailable; round results use the common cue"
                );
            }
            s2sdk::OnMapEnd_Register(on_map_end);
            s2sdk::OnMapStart_Register(on_map_start);
            s2sdk::OnEntityDeleted_Register(on_entity_deleted);
            s2sdk::OnGameFrame_Register(on_game_frame);
            if precache {
                s2sdk::OnBuildGameSessionManifest_Register(on_manifest);
            }
            for &event in EVENTS {
                let result = s2sdk::HookEvent(&Str::from_str(event), on_event, event_mode(event));
                if result == EventHookError::Okay {
                    with_state(|state| state.hooks.push(event));
                } else {
                    plugify::warning!("imomoe_sounds: cannot observe {}: {:?}", event, result);
                }
            }
            let command = Str::from_str(SOUND_COMMAND);
            let registered = !s2sdk::HasConsoleCommandCallbacks(&command)
                && s2sdk::AddConsoleCommand(
                    &command,
                    &Str::from_str("Toggle extra announcements"),
                    ConVarFlag::ClientCanExecute,
                    on_sound_command,
                    HookMode::Pre,
                );
            with_state(|state| state.sound_registered = registered);
            if !registered {
                plugify::warning!(
                    "imomoe_sounds: !sound is already in use or could not be registered"
                );
            }
            Ok(())
        },
    )
}

fn event_mode(name: &str) -> HookMode {
    if name == "round_prestart" {
        HookMode::Pre
    } else {
        HookMode::Post
    }
}

fn end() -> Result<(), Box<dyn Error>> {
    boundary("end", Err("imomoe_sounds: cleanup panicked".into()), || {
        let Some(mut state) = STATE.lock().unwrap_or_else(|e| e.into_inner()).take() else {
            return Ok(());
        };
        if state.precache {
            s2sdk::OnBuildGameSessionManifest_Unregister(on_manifest);
        }
        s2sdk::OnMapStart_Unregister(on_map_start);
        s2sdk::OnMapEnd_Unregister(on_map_end);
        s2sdk::OnEntityDeleted_Unregister(on_entity_deleted);
        s2sdk::OnGameFrame_Unregister(on_game_frame);
        if state.sound_registered {
            s2sdk::RemoveCommand(&Str::from_str(SOUND_COMMAND), on_sound_command);
        }
        for event in state.hooks {
            let result = s2sdk::UnhookEvent(&Str::from_str(event), on_event, event_mode(event));
            if !matches!(
                result,
                EventHookError::Okay | EventHookError::NotActive | EventHookError::InvalidCallback
            ) {
                plugify::error!("imomoe_sounds: cannot remove {}: {:?}", event, result);
            }
        }
        if state.live {
            state.playback.stop_all();
        }
        if let Err(error) = state.preferences.flush() {
            plugify::error!(
                "imomoe_sounds: cannot save player_preferences.json: {}",
                error
            );
        }
        Ok(())
    })
}

unsafe extern "C" fn on_map_start() {
    boundary("map start", (), || {
        with_state(|state| state.live = true);
    });
}

fn refresh_observer_schema() {
    let observer_schema = probe_observer_schema();
    let life_state_offset = probe_life_state_offset();
    let observer_pawn_offset = probe_observer_pawn_offset();
    with_state(|state| {
        state.observer_schema = observer_schema;
        state.life_state_offset = life_state_offset;
        state.observer_pawn_offset = observer_pawn_offset;
    });
}

unsafe extern "C" fn on_map_end() {
    boundary("map end", (), || {
        let result = with_state(|state| {
            state.live = false;
            state.combat = Combat::default();
            state.warmup_combat = Combat::default();
            state.pending_round_start = None;
            state.special_round_bgm_tick = None;
            state.observer_schema = None;
            state.life_state_offset = None;
            state.observer_pawn_offset = None;
            state.playback.reset_map();
            state.preferences.flush()
        });
        if let Some(Err(error)) = result {
            plugify::error!(
                "imomoe_sounds: cannot save player_preferences.json: {}",
                error
            );
        }
    });
}

unsafe extern "C" fn on_entity_deleted(handle: i32) {
    boundary("entity deleted", (), || {
        with_state(|state| state.playback.forget_entity(handle));
    });
}

unsafe extern "C" fn on_game_frame(_: bool, _: bool, _: bool) {
    boundary("game frame", (), || {
        let Some((batches, queued)) = with_state(|state| {
            let current_tick = s2sdk::GetGameTickCount();
            let formal: Vec<_> = state
                .combat
                .take_pending_before(current_tick)
                .into_iter()
                .map(|batch| (false, batch))
                .collect();
            let warmup: Vec<_> = state
                .warmup_combat
                .take_pending_before(current_tick)
                .into_iter()
                .map(|batch| (true, batch))
                .collect();
            let pending = state.pending_round_start.take();
            let replacement = replaces_round_start(pending, state.special_round_bgm_tick);
            if state.special_round_bgm_tick != pending {
                state.special_round_bgm_tick = None;
            }
            (
                formal.into_iter().chain(warmup).collect::<Vec<_>>(),
                (pending.is_some(), replacement),
            )
        }) else {
            return;
        };
        for (warmup, (slot, tick, cues)) in batches {
            let had_teamkill = cues.iter().any(|cue| cue == "teamkill");
            if let Some(selected) = playback::dispatch_selected(
                cues.iter().map(String::as_str),
                Target::Event(Some(slot)),
            ) {
                with_state(|state| {
                    let combat = if warmup {
                        &mut state.warmup_combat
                    } else {
                        &mut state.combat
                    };
                    combat.emitted(slot, tick, selected != "teamkill");
                    if had_teamkill || selected == "teamkill" {
                        combat.emitted_teamkill(slot, tick);
                    }
                });
            }
        }
        if queued.0 && !queued.1 {
            dispatch(["round_start"], Target::Event(None));
        }
    });
}

unsafe extern "C" fn on_manifest() {
    boundary("game session manifest", (), || {
        let Some(config) = with_state(|state| state.playback.config.clone()) else {
            return;
        };
        let ready = playback::precache(&config);
        with_state(|state| state.playback.mounted = ready);
    });
}

fn steam_id(slot: i32) -> Option<u64> {
    if !(0..s2sdk::GetMaxClients()).contains(&slot)
        || !s2sdk::IsClientInGame(slot)
        || s2sdk::IsFakeClient(slot)
    {
        return None;
    }
    let id = s2sdk::GetClientSteamID64(slot);
    (id != 0).then_some(id)
}

fn reply(slot: i32, context: ConCommandContext, chinese: String, english: String) {
    let language = s2sdk::GetClientLanguageCode(slot);
    let msg = if language.as_str().starts_with("zh") {
        chinese
    } else {
        english
    };
    let marker = with_state(|state| state.playback.config.chat_prefix_marker.clone()).flatten();
    let msg = chat_prefix::for_reply(slot, context, marker.as_deref(), &msg);
    s2sdk::ReplyToCommand(context, slot, &Str::from_str(&msg));
}

unsafe extern "C" fn on_sound_command(
    slot: i32,
    context: ConCommandContext,
    _: &Arr<Str>,
) -> ResultType {
    boundary("!sound", ResultType::Continue, || {
        let Some(steam) = steam_id(slot) else {
            return ResultType::Continue;
        };
        let Some((enabled, save)) = with_state(|state| state.preferences.toggle(steam)) else {
            return ResultType::Continue;
        };
        if let Err(error) = &save {
            plugify::error!(
                "imomoe_sounds: cannot save player_preferences.json: {}",
                error
            );
        }
        let (zh, en) = if enabled {
            ("已开启播报音效", "Announcements enabled.")
        } else {
            ("已关闭播报音效", "Announcements muted.")
        };
        let (zh, en) = if save.is_ok() {
            (zh.to_owned(), en.to_owned())
        } else {
            (
                format!("{zh}（本次会话；保存失败）"),
                format!("{en} (this session; saving failed)"),
            )
        };
        reply(slot, context, zh, en);
        ResultType::Handled
    })
}

fn player_slot(event: usize, key: &str) -> i32 {
    s2sdk::GetEventPlayerSlot(event, &Str::from_str(key))
}

fn playing_team(slot: i32, max: i32) -> Option<CSTeam> {
    if !(0..max).contains(&slot) {
        return None;
    }
    let team = s2sdk::GetClientTeam(slot);
    matches!(team, CSTeam::T | CSTeam::CT).then_some(team)
}

fn warmup() -> Option<bool> {
    let rules = s2sdk::GetGameRules();
    let class = Str::from_str("CCSGameRules");
    let field = Str::from_str("m_bWarmupPeriod");
    if rules == 0 || s2sdk::GetSchemaOffset(&class, &field) < 0 {
        return None;
    }
    Some(s2sdk::GetEntSchema2(rules, &class, &field, 0) != 0)
}

fn probe_observer_schema() -> Option<ObserverSchema> {
    let offset = |class: &str, field: &str| {
        s2sdk::GetSchemaOffset(&Str::from_str(class), &Str::from_str(field))
    };
    let schema = ObserverSchema {
        controller_observer_pawn: offset("CCSPlayerController", "m_hObserverPawn"),
        pawn_observer_services: offset("CBasePlayerPawn", "m_pObserverServices"),
        observer_mode: offset("CPlayer_ObserverServices", "m_iObserverMode"),
        observer_target: offset("CPlayer_ObserverServices", "m_hObserverTarget"),
    };
    (schema.controller_observer_pawn >= 0
        && schema.pawn_observer_services >= 0
        && schema.observer_mode >= 0
        && schema.observer_target >= 0)
        .then_some(schema)
}

fn probe_life_state_offset() -> Option<i32> {
    let offset =
        s2sdk::GetSchemaOffset(&Str::from_str("CBaseEntity"), &Str::from_str("m_lifeState"));
    (offset >= 0).then_some(offset)
}

fn probe_observer_pawn_offset() -> Option<i32> {
    let offset = s2sdk::GetSchemaOffset(
        &Str::from_str("CCSPlayerController"),
        &Str::from_str("m_hObserverPawn"),
    );
    (offset >= 0).then_some(offset)
}

unsafe fn read_schema<T: Copy>(entity: usize, offset: i32) -> T {
    unsafe { std::ptr::read_unaligned((entity as *const u8).add(offset as usize).cast::<T>()) }
}

fn observer_target_origin(slot: i32, schema: ObserverSchema) -> Option<plugify::Vec3> {
    // S2SDK's GetController(CPlayerSlot) resolves the controller at slot + 1.
    let controller_handle = s2sdk::EntIndexToEntHandle(slot + 1);
    if !s2sdk::IsValidEntHandle(controller_handle) {
        return None;
    }
    let controller = s2sdk::EntHandleToEntPointer(controller_handle);
    if controller == 0 {
        return None;
    }
    let observer_pawn_handle =
        unsafe { read_schema::<i32>(controller, schema.controller_observer_pawn) };
    if !s2sdk::IsValidEntHandle(observer_pawn_handle) {
        return None;
    }
    let observer_pawn = s2sdk::EntHandleToEntPointer(observer_pawn_handle);
    if observer_pawn == 0 {
        return None;
    }
    let services = unsafe { read_schema::<usize>(observer_pawn, schema.pawn_observer_services) };
    if services == 0 || unsafe { read_schema::<u8>(services, schema.observer_mode) } != 2 {
        return None;
    }
    let target = unsafe { read_schema::<i32>(services, schema.observer_target) };
    s2sdk::IsValidEntHandle(target).then(|| s2sdk::GetEntityAbsOrigin(target))
}

pub(crate) fn listener_origin(
    slot: i32,
    observer_pawn_offset: Option<i32>,
    life_state_offset: Option<i32>,
    observer_schema: Option<ObserverSchema>,
) -> Option<plugify::Vec3> {
    let pawn = s2sdk::GetClientPawn(slot);
    if pawn != 0 {
        // GetClientPawn returns controller.m_hPawn, which can be CCSObserverPawn.
        let controller_handle = s2sdk::EntIndexToEntHandle(slot + 1);
        if s2sdk::IsValidEntHandle(controller_handle) {
            let controller = s2sdk::EntHandleToEntPointer(controller_handle);
            if controller != 0 {
                let is_observer_pawn = observer_pawn_offset.is_some_and(|offset| {
                    let observer = unsafe { read_schema::<i32>(controller, offset) };
                    s2sdk::IsValidEntHandle(observer)
                        && s2sdk::EntHandleToEntPointer(observer) == pawn
                });
                let alive = life_state_offset.map_or_else(
                    || s2sdk::IsClientAlive(slot),
                    |offset| unsafe { read_schema::<u8>(pawn, offset) == 0 },
                );
                if !is_observer_pawn && alive {
                    return Some(s2sdk::GetClientAbsOrigin(slot));
                }
            }
        }
    }
    observer_target_origin(slot, observer_schema?)
}

fn replaces_round_start(pending: Option<i32>, replacement: Option<i32>) -> bool {
    pending.is_some_and(|tick| replacement == Some(tick))
}

fn round_result_cue(config: &Config, winner: i32, reason: i32) -> &'static str {
    // SDK CSRoundEndReason::Draw = 10; winner 1 also occurs at GameStart.
    let specific = match (winner, reason) {
        (2, _) => "round_end.t",
        (3, _) => "round_end.ct",
        (0 | 1, 10) => "round_end.draw",
        _ => return "round_end",
    };
    if config.playlists.contains_key(specific) {
        specific
    } else {
        "round_end"
    }
}

fn announce_death(event: usize) {
    let warmup_death = warmup() != Some(false);
    let victim = player_slot(event, "userid");
    let attacker = player_slot(event, "attacker");
    let max = s2sdk::GetMaxClients();
    let Some(victim_team) = playing_team(victim, max) else {
        return;
    };
    let team = playing_team(attacker, max);
    let same_team = team == Some(victim_team);
    let ffa = same_team
        && with_state(|state| state.teammates_are_enemies)
            .is_some_and(|handle| handle as u16 != u16::MAX && s2sdk::GetConVarBool(handle));
    let weapon = s2sdk::GetEventString(event, &Str::from_str("weapon"));
    let headshot = s2sdk::GetEventBool(event, &Str::from_str("headshot"));
    let death = classify(
        victim,
        team.map(|_| attacker),
        same_team,
        ffa,
        weapon.as_str(),
        headshot,
    );
    let tick = s2sdk::GetGameTickCount();
    let Some((actor, names, cue_kind)) = with_state(|state| {
        let combat = if warmup_death {
            &mut state.warmup_combat
        } else {
            &mut state.combat
        };
        combat.death(
            victim,
            death,
            tick,
            state.playback.config.shutdown_min_streak,
        )
    }) else {
        return;
    };
    if matches!(cue_kind, combat::CueKind::Enemy | combat::CueKind::Teamkill) {
        with_state(|state| {
            let combat = if warmup_death {
                &mut state.warmup_combat
            } else {
                &mut state.combat
            };
            combat.queue_death_cues(actor, tick, names);
        });
    } else if !names.is_empty() {
        dispatch(names.iter().map(String::as_str), Target::Event(Some(actor)));
    }
}

unsafe extern "C" fn on_event(name: &Str, event: usize, _: bool) -> ResultType {
    boundary("game event", ResultType::Continue, || {
        if event == 0 || with_state(|_| ()).is_none() {
            return ResultType::Continue;
        }
        match name.as_str() {
            "round_prestart" => {
                let tracks = with_state(|state| {
                    state.combat.new_round();
                    state.warmup_combat.new_round();
                    state.pending_round_start = None;
                    state.special_round_bgm_tick = None;
                    state.playback.take_interruptible()
                })
                .unwrap_or_default();
                playback::stop_world(tracks);
            }
            "round_start" => {
                refresh_observer_schema();
                with_state(|state| state.live = true);
                let tick = s2sdk::GetGameTickCount();
                with_state(|state| state.pending_round_start = Some(tick));
            }
            "round_freeze_end" => {
                dispatch(["freeze_end"], Target::Event(None));
            }
            "round_end" => {
                let specific =
                    if unsafe { std::ptr::addr_of!(s2sdk::__s2sdk_GetEventInt).read() }.is_some() {
                        let winner = s2sdk::GetEventInt(event, &Str::from_str("winner"));
                        let reason = s2sdk::GetEventInt(event, &Str::from_str("reason"));
                        with_state(|state| round_result_cue(&state.playback.config, winner, reason))
                    } else {
                        None
                    };
                dispatch([specific.unwrap_or("round_end")], Target::Event(None));
            }
            "player_connect_full" => {
                let slot = player_slot(event, "userid");
                if (0..s2sdk::GetMaxClients()).contains(&slot)
                    && s2sdk::IsClientInGame(slot)
                    && !s2sdk::IsFakeClient(slot)
                {
                    dispatch(["join_server"], Target::Event(Some(slot)));
                }
            }
            "player_death" => announce_death(event),
            "round_mvp" => {
                dispatch(["mvp"], Target::Event(Some(player_slot(event, "userid"))));
            }
            "player_disconnect" => {
                let slot = player_slot(event, "userid");
                with_state(|state| state.combat.disconnect(slot));
            }
            _ => {}
        }
        ResultType::Continue
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn PlayCue(cue: &Str, slot: i32) -> bool {
    boundary("PlayCue", false, || {
        dispatch([cue.as_str()], Target::Client(slot))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn BroadcastCue(cue: &Str, team: i32) -> bool {
    boundary("BroadcastCue", false, || {
        dispatch([cue.as_str()], Target::Team(team))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn BroadcastRoundStartCue(cue: &Str) -> bool {
    boundary("BroadcastRoundStartCue", false, || {
        let played = dispatch([cue.as_str()], Target::Team(0));
        if played {
            let tick = s2sdk::GetGameTickCount();
            with_state(|state| state.special_round_bgm_tick = Some(tick));
        }
        played
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn PlaySpatialCue(
    cue: &Str,
    entity_handle: i32,
    distance_cap: f32,
    tactical: bool,
) -> bool {
    boundary("PlaySpatialCue", false, || {
        dispatch(
            [cue.as_str()],
            Target::Spatial {
                source: entity_handle,
                distance_cap,
                tactical,
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn StopCue(cue: &Str, entity_handle: i32) -> bool {
    boundary("StopCue", false, || {
        playback::stop_spatial(cue.as_str(), entity_handle)
    })
}

/// Reports whether a tactical cue currently has an available clip.
#[unsafe(no_mangle)]
pub extern "C" fn HasTacticalCue(cue: &Str) -> bool {
    boundary("HasTacticalCue", false, || {
        with_state(|state| state.playback.tactical_ready(cue.as_str())).unwrap_or(false)
    })
}

register_plugin!(start: start, end: end);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    static EMISSIONS: Mutex<Vec<(String, Vec<i32>)>> = Mutex::new(Vec::new());

    unsafe extern "C" fn max_clients() -> i32 {
        5
    }
    unsafe extern "C" fn in_game(slot: i32) -> bool {
        (0..4).contains(&slot)
    }
    unsafe extern "C" fn fake_client(slot: i32) -> bool {
        slot == 3
    }
    unsafe extern "C" fn source_tv(_: i32) -> bool {
        false
    }
    unsafe extern "C" fn steam_id(slot: i32) -> u64 {
        1001 + slot as u64
    }
    unsafe extern "C" fn event_slot(event: usize, _: &Str) -> i32 {
        event as i32 - 10
    }
    unsafe extern "C" fn event_int(event: usize, key: &Str) -> i32 {
        if key.as_str() == "reason" {
            (event / 100) as i32
        } else {
            (event % 10) as i32
        }
    }
    unsafe extern "C" fn entity_handle(_: i32) -> i32 {
        0
    }
    unsafe extern "C" fn emit(_: i32, slots: &Arr<i32>, cue: &Str, _: f32, _: f32) {
        EMISSIONS
            .lock()
            .unwrap()
            .push((cue.as_str().to_owned(), slots.iter().copied().collect()));
    }

    #[test]
    fn connect_and_round_results_follow_audience_mute_and_configured_fallback() {
        crate::test_support::init_plugify_values();
        unsafe {
            s2sdk::__s2sdk_GetMaxClients = Some(max_clients);
            s2sdk::__s2sdk_IsClientInGame = Some(in_game);
            s2sdk::__s2sdk_IsFakeClient = Some(fake_client);
            s2sdk::__s2sdk_IsClientSourceTV = Some(source_tv);
            s2sdk::__s2sdk_GetClientSteamID64 = Some(steam_id);
            s2sdk::__s2sdk_GetEventPlayerSlot = Some(event_slot);
            s2sdk::__s2sdk_GetEventInt = Some(event_int);
            s2sdk::__s2sdk_EntIndexToEntHandle = Some(entity_handle);
            s2sdk::__s2sdk_EmitSoundToClient = Some(emit);
        }
        let dir = tempfile::tempdir().unwrap();
        let config = Config::parse(
            r#"{"announcements":{
            "join_server":{"audience":"actor","clips":["Join"]},
            "round_end":{"clips":["Generic"]},
            "round_end.ct":{"clips":["CT"]},
            "round_end.t":{"clips":[]}
        }}"#,
        )
        .unwrap();
        let mut preferences = Preferences::new(dir.path().join("prefs.json"), true);
        preferences.toggle(1002).1.unwrap();
        *STATE.lock().unwrap() = Some(State {
            thread: thread::current().id(),
            playback: Playback::new(config).unwrap(),
            preferences,
            combat: Combat::default(),
            warmup_combat: Combat::default(),
            teammates_are_enemies: 0,
            live: true,
            hooks: Vec::new(),
            sound_registered: false,
            precache: false,
            pending_round_start: None,
            special_round_bgm_tick: None,
            observer_schema: None,
            life_state_offset: None,
            observer_pawn_offset: None,
        });
        let fire = |name: &str, event| unsafe { on_event(&Str::from_str(name), event, false) };
        EMISSIONS.lock().unwrap().clear();
        fire("player_connect_full", 10); // Human, actor-only.
        fire("player_connect_full", 11); // Muted human.
        fire("player_connect_full", 13); // BOT.
        fire("player_connect_full", 14); // Disconnected slot.
        assert_eq!(*EMISSIONS.lock().unwrap(), [("Join".into(), vec![0])]);

        let all = Config::parse(
            r#"{"announcements":{
            "join_server":{"audience":"all","clips":["JoinAll"]},
            "round_end":{"clips":["Generic"]},
            "round_end.ct":{"clips":["CT"]},
            "round_end.t":{"clips":[]}
        }}"#,
        )
        .unwrap();
        with_state(|state| state.playback.config = Arc::new(all));
        EMISSIONS.lock().unwrap().clear();
        fire("player_connect_full", 10);
        fire("round_end", 13); // CT-specific.
        fire("round_end", 12); // Explicitly empty T cue stays silent.
        fire("round_end", 1011); // Omitted draw cue uses common cue.
        assert!(!dispatch(["round_end.t", "round_end"], Target::Event(None)));
        assert_eq!(
            *EMISSIONS.lock().unwrap(),
            [
                ("JoinAll".into(), vec![0, 2]),
                ("CT".into(), vec![0, 2]),
                ("Generic".into(), vec![0, 2])
            ]
        );
        let draw = Config::parse(
            r#"{"announcements":{
            "round_end":{"clips":["Generic"]},
            "round_end.draw":{"clips":["Draw"]}
        }}"#,
        )
        .unwrap();
        with_state(|state| state.playback.config = Arc::new(draw));
        EMISSIONS.lock().unwrap().clear();
        fire("round_end", 1010); // No team, Draw.
        fire("round_end", 1011); // Unassigned team, Draw.
        fire("round_end", 1611); // GameStart, not a draw.
        fire("round_end", 1111); // Unknown no-winner reason.
        assert_eq!(
            *EMISSIONS.lock().unwrap(),
            [
                ("Draw".into(), vec![0, 2]),
                ("Draw".into(), vec![0, 2]),
                ("Generic".into(), vec![0, 2]),
                ("Generic".into(), vec![0, 2])
            ]
        );
        STATE.lock().unwrap().take();
        unsafe {
            s2sdk::__s2sdk_GetMaxClients = None;
            s2sdk::__s2sdk_IsClientInGame = None;
            s2sdk::__s2sdk_IsFakeClient = None;
            s2sdk::__s2sdk_IsClientSourceTV = None;
            s2sdk::__s2sdk_GetClientSteamID64 = None;
            s2sdk::__s2sdk_GetEventPlayerSlot = None;
            s2sdk::__s2sdk_GetEventInt = None;
            s2sdk::__s2sdk_EntIndexToEntHandle = None;
            s2sdk::__s2sdk_EmitSoundToClient = None;
        }
    }

    #[test]
    fn missing_is_client_alive_binding_prevents_startup() {
        let error = require_bindings(false).unwrap_err();
        assert_eq!(
            error.to_string(),
            "missing S2SDK binding __s2sdk_IsClientAlive"
        );
    }

    #[test]
    fn special_bgm_replaces_only_round_start_from_same_tick() {
        assert!(replaces_round_start(Some(12), Some(12)));
        assert!(!replaces_round_start(Some(12), Some(13)));
        assert!(!replaces_round_start(Some(12), None));
        assert!(!replaces_round_start(None, Some(12)));
    }
}
