<div align="center">

# Imomoe Sounds (`imomoe_sounds`)

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Rust: 1.99.1](https://img.shields.io/badge/Rust-1.99.1%20(2024)-orange.svg)](rust-toolchain.toml)
[![CS2: Compatible](https://img.shields.io/badge/CS2-Dedicated%20Server-black.svg)](https://www.counter-strike.net/)
[![Platform: Plugify](https://img.shields.io/badge/Plugify-v3.1.0%2B-purple.svg)](https://github.com/untrustedmodders/plugify)
[![CI](https://github.com/Imomoe-Community/Imomoe-Sounds/actions/workflows/ci.yml/badge.svg)](https://github.com/Imomoe-Community/Imomoe-Sounds/actions/workflows/ci.yml)

**面向 Counter-Strike 2 专用服务端的 Plugify 战斗播报、空间战术提示与个人音效偏好插件**<br>
<sub><sup>A Plugify plugin for CS2 dedicated servers providing combat announcements, spatial tactical cues, Workshop precaching, and per-player sound toggles (`!sound`).</sup></sub>

**双语 / Bilingual** · [English](README.en.md)

</div>

---

### 项目简介 / Overview

**imomoe_sounds** 是面向 Counter-Strike 2 (CS2) 专用服务端的独立 Rust [Plugify](https://github.com/untrustedmodders/plugify) 音效插件。负责战斗与回合事件播报、Workshop 自定义声效资源预缓存、3D 空间战术提示以及玩家个人播报开关（`!sound`）；玩法模组（如 FunRounds）只需按需调用导出接口触发对应 Cue。
<sub><sup>**imomoe_sounds** is an independent Rust [Plugify](https://github.com/untrustedmodders/plugify) plugin for CS2 dedicated servers handling combat and round announcements, Workshop SoundEvent precaching, 3D spatial tactical cues, and personal `!sound` toggles. Gameplay plugins such as FunRounds trigger cues cleanly via exported ABI methods.</sup></sub>

---

### 环境要求 / Prerequisites

- CS2 Dedicated Server 与兼容的 Plugify（`v3.1.0+`）<br><sub><sup>CS2 Dedicated Server with compatible Plugify (`v3.1.0+`)</sup></sub>
- Plugify Rust Module（`1.3.0`，若使用 `1.2.5` 请注意 Linux 产物文件名约定）<br><sub><sup>Plugify Rust Module (`1.3.0`, or `1.2.5` with the Linux filename convention)</sup></sub>
- S2SDK `>=3.4.2`（若使用裁剪过的 S2SDK provider，请确保导出清单包含 [bindings/s2sdk.pplugin](bindings/s2sdk.pplugin) 声明的方法与原型）<br><sub><sup>S2SDK `>=3.4.2` (trimmed providers must retain all methods and prototypes in [bindings/s2sdk.pplugin](bindings/s2sdk.pplugin))</sup></sub>
- （可选）[Imomoe Chat Prefix](https://github.com/Imomoe-Community/Imomoe-Chat-Prefix)（`imomoe_chat_prefix >=0.1.0`），用于统一 `!sound` 聊天回复前缀<br><sub><sup>(Optional) [Imomoe Chat Prefix](https://github.com/Imomoe-Community/Imomoe-Chat-Prefix) (`>=0.1.0`) for `!sound` chat reply prefixes</sup></sub>

---

### 安装与部署 / Installation

1. 构建发布产物 / Build release binaries:

   ```bash
   cargo build --release --locked
   ```

2. 将 [imomoe_sounds.pplugin](imomoe_sounds.pplugin) 与编译产物放入 `game/csgo/addons/plugify/envs/imomoe_sounds/` / Deploy manifest and binary:
   - **Windows**：将 `target/release/imomoe_sounds.dll` 放入 `bin/imomoe_sounds.dll`。
   - **Linux**：将 `target/release/libimomoe_sounds.so` 放入 `bin/libimomoe_sounds.so`。

3. 创建运行时配置目录 `game/csgo/addons/plugify/configs/imomoe_sounds/` / Prepare configuration directory:
   - 将 [schemas/sounds.schema.json](configs/imomoe_sounds/schemas/sounds.schema.json) 复制到该目录的 `schemas/` 下（用于编辑器校验与自动补全）。<br><sub><sup>Copy `sounds.schema.json` into `schemas/` for editor validation and autocompletion.</sup></sub>
   - 将 [sounds.example.jsonc](configs/imomoe_sounds/sounds.example.jsonc)（原生 CS2 音效预设，无需 Workshop）或 [sounds.workshop.example.jsonc](configs/imomoe_sounds/sounds.workshop.example.jsonc)（Workshop 自定义音效预设）复制为 `sounds.jsonc` 并按需修改。<br><sub><sup>Copy `sounds.example.jsonc` or `sounds.workshop.example.jsonc` to `sounds.jsonc` and customize.</sup></sub>

4. 重启服务端或按运维流程重新加载插件使其生效。<br><sub><sup>Restart or reload the plugin to apply changes.</sup></sub>

---

### 配置播放与收听 / Configuration & Playback

配置文件支持 JSONC 注释与尾随逗号，并通过 [sounds.schema.json](configs/imomoe_sounds/schemas/sounds.schema.json) 进行严格校验（不允许未知字段）：<br><sub><sup>Configs support JSONC comments/trailing commas and are strictly validated against [sounds.schema.json](configs/imomoe_sounds/schemas/sounds.schema.json):</sup></sub>

- **播放策略 / Playback Order**：`order` 支持 `sequential`（全服同图按序循环，换图复位）、`actor_sequential`（按触发玩家或发声实体独立轮播；玩家阵亡或断线、实体删除、换图时复位）与 `random`（等概率独立抽取）。省略配置项将跳过该候选，显式配置 `clips: []` 则静音。<br><sub><sup>`order` supports `sequential` (map-wide), `actor_sequential` (per player life or entity), and `random`. Empty `clips: []` mutes the cue.</sup></sub>
- **`announcements`**：配置回合、MVP 与战斗播报。支持通过 `combo_window_seconds`（默认 `2.0`，设 `0` 关闭）配置限时连杀 `combo.N`，以及通过 `headshot_streak.N` 配置回合内连续爆头（未命中具体次数时回退通用 `headshot` 池）。非 FFA 误杀（`teamkill`）与自杀（`suicide`）独立触发。<br><sub><sup>Configures round, MVP, and combat announcements, including timed `combo.N`, `headshot_streak.N` (with `headshot` fallback), `teamkill`, and `suicide`.</sup></sub>
- **击杀互斥开关 / Exclusive Kill ConVar (`imomoe_sounds_exclusive`)**：运行时 ConVar（默认 `1`）控制击杀播报是否互斥。设为 `1` 时按 `first_blood` → `combo.N` → `streak.N` → `streak_shutdown` → `weapons.<name>` → `headshot_streak.N` → `headshot` → `kill` 仅播优先级最高的一条；设为 `0` 时按维度叠加播放（同一 Tick 同维度至多一条），两种模式下全部计数均正常推进。<br><sub><sup>`1` (default) emits only the highest-priority kill cue; `0` emits at most one cue per dimension per tick; all counters advance in both modes.</sup></sub>
- **`announcements.join_server` 与 `round_end.*`**：`join_server` 可设置 `audience` 为 `"actor"` 或 `"all"`；回合结算支持 `round_end.ct`、`round_end.t` 与 `round_end.draw`（仅无胜方且结算原因为 Draw 时触发），省略结果键时回退至公共 `round_end`，设 `clips: []` 则静音。<br><sub><sup>`join_server` supports `"actor"`/`"all"` audience; `round_end` routes `.ct`, `.t`, and `.draw` with fallback to `round_end`.</sup></sub>
- **`funrounds.flavor`**：模式 BGM 与玩家定向效果音；普通模式 BGM 支持 `"interruptible": true`，在下一回合 `round_prestart` 时自动停止。<br><sub><sup>Mode BGM and targeted flavor cues; `"interruptible": true` stops on the next `round_prestart`.</sup></sub>
- **`funrounds.tactical`**：实体空间战术提示（如幽灵呼吸、自爆警告与爆炸），按 `distance` 筛选且不受 `!sound` 个人静音影响；空间衰减由 SoundEvent 自身定义负责。<br><sub><sup>3D entity tactical cues filtered by `distance`; unaffected by `!sound` mute.</sup></sub>
- **`workshop`**：仅使用自定义音效时需要。填写 `id`、`events`（`.vsndevts`）与 `resources`（`.vsnd`），并将物品 ID 加入服务端 `s2_extra_addons`。详见 [Workshop 指南](docs/WORKSHOP.md)。<br><sub><sup>For custom Workshop assets via `s2_extra_addons`; see the [Workshop Guide](docs/WORKSHOP.md#english).</sup></sub>

#### 玩家开关与聊天前缀 / Player Toggle & Chat Prefix

- 玩家在聊天或控制台输入 `!sound` 切换个人播报开关。偏好以 SteamID64 为键即时原子写入 `player_preferences.json`，仅保存与服务端 `enabled_by_default` 不同的设置。<br><sub><sup>`!sound` toggles personal announcements and persists non-default preferences atomically by SteamID64 to `player_preferences.json`.</sup></sub>
- 配置 `chat_prefix_marker`（如 `"<prefix>"`）且安装 `imomoe_chat_prefix` 时，`!sound` 聊天回复会自动注入当前社区前缀；前缀插件缺失时自动移除标记与分隔空格，控制台回复始终保持纯文本。<br><sub><sup>When `chat_prefix_marker` is set and `imomoe_chat_prefix` is loaded, `!sound` chat replies prepend the shared prefix; console replies remain plain text.</sup></sub>

---

### Plugify API

在游戏主线程传入配置中的 Cue 标识（例如 `kill`、`funrounds.flavor.knife_round_start` 或 `funrounds.tactical.ghost_breath`） / Invoke on the game thread with a configured cue key:

| 方法 / <sub><sup>Method</sup></sub> | 说明 / <sub><sup>Description</sup></sub> |
| :--- | :--- |
| `PlayCue(cue, slot)` | 向指定玩家槽位播放 Cue，尊重个人 `!sound` 设置。<br><sub><sup>Plays a cue to an eligible player slot, respecting `!sound`.</sup></sub> |
| `BroadcastCue(cue, team)` | 向 `0` 全服（含观察者与 CSTV）、`2` T 或 `3` CT 播报，尊重真人玩家设置。<br><sub><sup>Broadcasts to `0` (all including spectators/CSTV), `2` (T), or `3` (CT).</sup></sub> |
| `BroadcastRoundStartCue(cue)` | 发出模式开局音轨；发声成功时替代同 Tick 的默认 `round_start` 播报。<br><sub><sup>Emits a mode opening track; replaces default `round_start` on that tick when emitted.</sup></sub> |
| `PlaySpatialCue(cue, entity_handle, distance_cap, tactical)` | 从有效完整实体句柄发声。`distance_cap` 为 `0` 时不加调用侧上限，正数则与配置 `distance` 取较小值；`tactical` 必须与 Cue 分类匹配。<br><sub><sup>Emits from a valid full entity handle (`distance_cap = 0` uses config `distance`); `tactical` must match cue category.</sup></sub> |
| `StopCue(cue, entity_handle)` | 停止该实体上此 Cue 最近一次实际播放的音轨。<br><sub><sup>Stops the active clip last started for that cue and entity.</sup></sub> |
| `HasTacticalCue(cue)` | 查询战术 Cue 当前是否至少有一条可用或已预缓存的音轨；不应用于阻断玩法逻辑。<br><sub><sup>Checks whether a tactical cue has a ready or precached clip; never gate gameplay logic on it.</sup></sub> |

对播放与停止方法，返回 `true` 表示已向引擎发出请求；无匹配 Cue、无合格听众或实体句柄无效时返回 `false`（不会回退到世界实体）。<br><sub><sup>Play/stop methods return `true` when emitted; missing cues, no eligible listeners, or invalid entity handles return `false` without falling back to world entity `0`.</sup></sub>

---

### 构建与开发 / Building & Development

本仓库使用 Rust 2024 Edition，工具链固定为 `1.99.1`（见 [rust-toolchain.toml](rust-toolchain.toml)）。提交改动前运行标准四门禁：<br><sub><sup>Uses Rust 2024 Edition pinned to `1.99.1`. Run all four checks before submitting changes:</sup></sub>

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

更新 [bindings/s2sdk.pplugin](bindings/s2sdk.pplugin) 后重新生成绑定 / Regenerate S2SDK bindings:

```bash
plugify-gen -lang rust -manifest bindings/s2sdk.pplugin -output . -overwrite
cargo fmt
```

- [架构说明 / Architecture](docs/ARCHITECTURE.md) · [Workshop 指南 / Workshop](docs/WORKSHOP.md) · [FunRounds 联动说明 / Integration](docs/FUNROUNDS_INTEGRATION.md)
- [验收测试指南 / Acceptance](docs/ACCEPTANCE.md) · [联动设计 RFC / RFC](docs/RFC_FUNROUNDS_INTEGRATION.md)
- [贡献指南 / Contributing](CONTRIBUTING.md) · [安全通告 / Security](SECURITY.md) · [版本记录 / Changelog](CHANGELOG.md)

---

### 开源协议与作者 / License & Author

- **作者 / Author**：Mai Ooizumi ([@Mai-Ooizumi](https://github.com/Mai-Ooizumi))
- **开源协议 / License**：GNU Affero General Public License v3.0 or later ([AGPL-3.0-or-later](LICENSE))
