<div align="center">

# Imomoe Sounds (`imomoe_sounds`)

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Rust: 1.99.1](https://img.shields.io/badge/Rust-1.99.1%20(2024)-orange.svg)](rust-toolchain.toml)
[![CS2: Compatible](https://img.shields.io/badge/CS2-Dedicated%20Server-black.svg)](https://www.counter-strike.net/)
[![Platform: Plugify](https://img.shields.io/badge/Plugify-v3.1.0%2B-purple.svg)](https://github.com/untrustedmodders/plugify)
[![CI](https://github.com/Imomoe-Community/Imomoe-Sounds/actions/workflows/ci.yml/badge.svg)](https://github.com/Imomoe-Community/Imomoe-Sounds/actions/workflows/ci.yml)

**A Plugify plugin for CS2 dedicated servers providing combat announcements, spatial tactical cues, Workshop precaching, and per-player sound toggles (`!sound`).**

[双语 / Bilingual](README.md) · **English**

</div>

---

### Overview

**imomoe_sounds** is an independent Rust [Plugify](https://github.com/untrustedmodders/plugify) plugin for Counter-Strike 2 dedicated servers. It manages combat and round announcements, Workshop custom SoundEvent precaching, 3D spatial tactical cues, and personal `!sound` toggles. Gameplay modes such as FunRounds trigger cues on demand through exported ABI calls.

---

### Prerequisites

- CS2 Dedicated Server with compatible Plugify (`v3.1.0+`)
- Plugify Rust Module (`1.3.0`; if using `1.2.5`, follow its Linux library filename convention)
- S2SDK `>=3.4.2` (trimmed S2SDK providers must include all methods and prototypes declared in [bindings/s2sdk.pplugin](bindings/s2sdk.pplugin))
- (Optional) [Imomoe Chat Prefix](https://github.com/Imomoe-Community/Imomoe-Chat-Prefix) (`imomoe_chat_prefix >=0.1.0`) for unified `!sound` chat reply prefixes

---

### Installation

1. Build release binaries:

   ```bash
   cargo build --release --locked
   ```

2. Deploy [imomoe_sounds.pplugin](imomoe_sounds.pplugin) and the binary into `game/csgo/addons/plugify/envs/imomoe_sounds/`:
   - **Windows**: `target/release/imomoe_sounds.dll` → `bin/imomoe_sounds.dll`
   - **Linux**: `target/release/libimomoe_sounds.so` → `bin/libimomoe_sounds.so`

3. Create `game/csgo/addons/plugify/configs/imomoe_sounds/`:
   - Copy [schemas/sounds.schema.json](configs/imomoe_sounds/schemas/sounds.schema.json) into `schemas/` for editor validation and autocompletion.
   - Copy [sounds.example.jsonc](configs/imomoe_sounds/sounds.example.jsonc) (built-in CS2 events, no Workshop required) or [sounds.workshop.example.jsonc](configs/imomoe_sounds/sounds.workshop.example.jsonc) (custom Workshop assets) to `sounds.jsonc` and edit as needed.

4. Restart the server or reload the plugin to take effect.

---

### Configuration & Playback

Configs support JSONC comments and trailing commas and are strictly validated against [sounds.schema.json](configs/imomoe_sounds/schemas/sounds.schema.json) (unknown fields rejected):

- **Playback Order**: `order` supports `sequential` (map-wide rotation, resets on map change), `actor_sequential` (independent rotation per player/entity, resets on disconnect/deletion or map change), and `random` (uniform independent draw). `clips` can mix built-in CS2 events and custom SoundEvents; omitting an entry skips that candidate, while `clips: []` explicitly mutes it.
- **`announcements`**: Configures round, MVP, and combat cues. Supports timed kill combos `combo.N` via `combo_window_seconds` (default `2.0`, `0` disables) and within-round consecutive headshots `headshot_streak.N` (falling back to `headshot` when a specific count is omitted). Non-FFA `teamkill` and `suicide` trigger independently.
- **Exclusive Kill ConVar (`imomoe_sounds_exclusive`)**: Runtime ConVar (default `1`). When `1`, emits only the highest-priority cue per tick (`first_blood` → `combo.N` → `streak.N` → `streak_shutdown` → `weapons.<name>` → `headshot_streak.N` → `headshot` → `kill`); when `0`, emits up to one cue per dimension per tick. All counters advance normally in both modes.
- **`announcements.join_server` & `round_end.*`**: `join_server` accepts `"actor"` or `"all"` audience; `round_end` supports `round_end.ct`, `round_end.t`, and `round_end.draw` (only when there is no winner and reason is Draw), falling back to shared `round_end` or muting on `clips: []`.
- **`funrounds.flavor`**: Mode BGM and targeted flavor cues; non-spatial BGM supports `"interruptible": true` to stop automatically on the next `round_prestart`.
- **`funrounds.tactical`**: 3D entity spatial cues (e.g. ghost breathing, suicide warning/explosion) filtered by `distance` and unaffected by `!sound` mute; distance attenuation is defined in the SoundEvent itself.
- **`workshop`**: Needed only for custom assets. Set `id`, `events` (`.vsndevts`), and `resources` (`.vsnd`), and add the Workshop ID to `s2_extra_addons`. See the [Workshop Guide](docs/WORKSHOP.md#english).

#### Player Toggle & Chat Prefix

- Players type `!sound` in chat or console to toggle personal announcements. Non-default settings are persisted atomically per SteamID64 to `player_preferences.json`.
- When `chat_prefix_marker` (e.g. `"<prefix>"`) is configured and `imomoe_chat_prefix` is installed, `!sound` chat replies prepend the current prefix; if absent, the marker and its separator space are stripped cleanly. Console replies stay plain text.

---

### Plugify API

Invoke on the game thread with a configured cue key (e.g. `kill`, `funrounds.flavor.knife_round_start`, `funrounds.tactical.ghost_breath`):

| Method | Description |
| :--- | :--- |
| `PlayCue(cue, slot)` | Plays a cue to an eligible player slot, respecting `!sound`. |
| `BroadcastCue(cue, team)` | Broadcasts to `0` (all including spectators and CSTV), `2` (T), or `3` (CT), respecting human preferences. |
| `BroadcastRoundStartCue(cue)` | Emits a mode opening track; replaces default `round_start` on that tick when emitted successfully. |
| `PlaySpatialCue(cue, entity_handle, distance_cap, tactical)` | Emits from a valid full entity handle (`distance_cap = 0` uses config `distance`; positive values take the minimum); `tactical` must match cue category. |
| `StopCue(cue, entity_handle)` | Stops the exact active clip last started for this cue and entity. |
| `HasTacticalCue(cue)` | Checks whether a tactical cue has a ready or precached clip; should not gate gameplay logic. |

Play/stop calls return `true` when emitted to the engine, or `false` on missing cues, no eligible listeners, or invalid entity handles (never falling back to world entity `0`).

---

### Building & Development

Uses Rust 2024 Edition pinned to `1.99.1` ([rust-toolchain.toml](rust-toolchain.toml)). Run all four checks before submitting changes:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

Regenerate bindings after updating [bindings/s2sdk.pplugin](bindings/s2sdk.pplugin):

```bash
plugify-gen -lang rust -manifest bindings/s2sdk.pplugin -output . -overwrite
cargo fmt
```

- [Architecture](docs/ARCHITECTURE.md#english) · [Workshop Guide](docs/WORKSHOP.md#english) · [FunRounds Integration](docs/FUNROUNDS_INTEGRATION.md#english)
- [Acceptance Guide](docs/ACCEPTANCE.md#english) · [Integration RFC](docs/RFC_FUNROUNDS_INTEGRATION.md#english)
- [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md) · [Changelog](CHANGELOG.md)

---

### License & Author

- **Author**: Mai Ooizumi ([@Mai-Ooizumi](https://github.com/Mai-Ooizumi))
- **License**: GNU Affero General Public License v3.0 or later ([AGPL-3.0-or-later](LICENSE))
