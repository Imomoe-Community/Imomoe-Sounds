# RFC: FunRounds 深度集成与多维音效系统架构规划 / Integration RFC

[简体中文](#简体中文) | [English](#english)

---

## 简体中文

> [!NOTE]
> 本文档为早期设计记录。后续演进已新增 `BroadcastRoundStartCue` 与 `HasTacticalCue` 导出方法，并移除了私有分发回退表与旧打包脚本；当前完整接口与实际配置请以 [README](../README.md#plugify-api) 及 [FunRounds 联动配置说明](FUNROUNDS_INTEGRATION.md) 为准。

本文档记录了 `imomoe_sounds` 音效插件与 `funrounds` 玩法模组深度解耦、接口补齐、全服音效统一治理及最终确认的工程实现细节。

---

### 1. 核心设计原则与已确认路线

1. **职责分离 (Separation of Concerns)**：
   - **FunRounds 玩法层**：专注于模式规则、生命周期与节拍调度（如幽灵呼吸间隔控制）、属性修改、实体交互与回合胜负判定，不内嵌具体音频资源路径。
   - **Imomoe Sounds 音效层**：集中管理全服所有声音资产、选曲策略、玩家偏好持久化、空间距离筛选、活跃音轨追踪、多样化音效池与自动预缓存。
2. **分发路线确定：路线 A（`s2_extra_addons` 默认下载 + 放弃双套 Fallback 维护）**：
   - 服务端将精简高压缩的创意工坊音效包加入公共 `s2_extra_addons`，玩家首次进服即完成自动下载与本地永久缓存，彻底消除人工维护 SteamID 白名单的负担。
   - 取消强制双套音效（工坊 + 原生 Fallback）的映射要求，新加武器或模式音效只需配置单一事件名。
   - 换图构建 Manifest 时保留 `ReadFileVPK` 校验与全量 `Precache`，防止对局中首次发声触发磁盘 I/O 卡顿。
3. **音效二元隔离 (Tactical vs Flavor)**：
   - **氛围与播报类 (Flavor / Broadcast)**：2D UI 音效（挂载在 `world` 实体 0），全局播报（含观察者与 CSTV），受玩家个人 `!sound` 控制。
   - **战术机制类 (Tactical / Spatial)**：3D 空间实体发声（挂载在 Pawn 或道具实体），属于核心对战机制（如幽灵呼吸、自爆引信），**严禁被个人指令静音**，保证对局公平性。
4. **多音效池与多样化 (Multi-Clip Variation Pools)**：
   - 每一类音效（无论是播报类还是战术类）均支持配置多个风格化或微差异的音频变体（Clips），支持 `random`（等概率随机抽取）与 `sequential`（顺序轮换），制作阶段统一响度。

---

### 2. 事件系统扩展与战斗逻辑

常规战斗事件在 CS2 引擎中原生具备完整上下文，由音效插件直接 Hook 捕获并处理：

- **回合节奏事件**：`round_prestart`（强制中断并冲刷上一回合未播完的 `interruptible` 长 BGM）、`round_start` / `round_freeze_end` / `round_end`。
- **第一滴血 (First Blood)**：回合重置后首次产生非自杀敌对击杀时触发，全服广播。
- **连杀与终结 (Streak & Shutdown)**：支持多级连杀播报，阵亡时连杀计数归零；**半场换边时保留连杀计数**；击杀高连杀者触发 `streak_shutdown`。
- **团队误杀与自杀 (Teamkill & Suicide)**：实时读取 `mp_teammates_are_enemies` 判定 FFA 模式；非 FFA 下同阵营误杀触发 `teamkill`，坠亡/自爆/世界伤害触发 `suicide`。
- **武器动态映射 (Dynamic Weapon Cues)**：读取 `player_death` 的 `weapon` 字符串动态检索 `weapons.<name>`。

#### 优先级裁决与同 Tick 防抖（方案 A）
- **同 Tick 同步防抖（零延迟）**：维护每个凶手最后发声的 `last_kill_tick`，同 Tick 多次击杀累加计数并避免叠加爆音。
- **优先级金字塔**：
  $$\text{First Blood (若未触发)} \longrightarrow \text{连杀 / 终结连杀} \longrightarrow \text{特定武器} \longrightarrow \text{爆头} \longrightarrow \text{普通击杀} \longrightarrow \text{TK / 自杀}$$

---

### 3. 客户端指令、CSTV 兼容与持久化设计

1. **Mixgroup 与个人播报开关**：非空间提示音归入 `mixgroup = "UI"`（不被 `snd_musicvolume 0` 屏蔽）；`!sound` 按 `GetClientLanguageCode(slot)` 返回中英文反馈。
2. **观察者与 CSTV 精确筛选**：全局广播包含真人玩家（`IsClientInGame && !IsFakeClient`）与 CSTV（`IsClientInGame && IsClientSourceTV`），跳过 T/CT AI BOT。
3. **零冗余偏好持久化**：`configs/imomoe_sounds/player_preferences.json` 以 SteamID64 为键仅存储非默认设置，同目录临时写入并原子替换。

---

### 4. 对外导出接口与工程契约 (Plugify ABI)

```rust
pub extern "C" fn PlayCue(cue: &Str, slot: i32) -> bool;
pub extern "C" fn BroadcastCue(cue: &Str, team: i32) -> bool;
pub extern "C" fn PlaySpatialCue(cue: &Str, entity_handle: i32, distance_cap: f32, tactical: bool) -> bool;
pub extern "C" fn StopCue(cue: &Str, entity_handle: i32) -> bool;
```

- **活跃音轨精确追踪**：内部维护 `active_entity_cues` 与 `active_world_interruptible`，在 `StopCue` 或 `round_prestart` 时传入实际播放的 `ActualClipName` 调用 `StopSound`。
- **空间发声实体校验**：入口强制校验 `s2sdk::IsValidEntHandle(entity_handle)`，失效时静默返回 `false`，不降级为世界实体。

---

### 5. 配置文件规范：JSONC + JSON Schema

默认配置采用支持注释与尾随逗号的 `configs/imomoe_sounds/sounds.jsonc`，配套 `sounds.schema.json` 提供静态校验与编辑器补全。

---

## English

> [!NOTE]
> Early architecture design record. Later updates added `BroadcastRoundStartCue` and `HasTacticalCue` exports and removed legacy per-client native fallback tables. Refer to [README.en.md](../README.en.md#plugify-api) and [FunRounds Integration](FUNROUNDS_INTEGRATION.md#english) for the current contract.

### 1. Core Principles

- **Separation of Concerns**: FunRounds owns round rules, entity identities, and timers; `imomoe_sounds` owns audio assets, clip rotation, SteamID64 preferences, spatial distance filtering, active clip tracking, and Workshop precaching.
- **Public Workshop Distribution**: Distributes compressed custom SoundEvents via `s2_extra_addons` with per-map `ReadFileVPK` validation and `Precache` without maintaining SteamID allowlists.
- **Tactical vs Flavor Isolation**: 2D UI announcements/BGM respect `!sound`; 3D tactical entity cues cannot be muted by `!sound`.

### 2. Combat Events, Priority & Persistence

- Hooks `round_prestart`, `round_start`, `round_freeze_end`, `round_end`, and `player_death`, resolving same-tick kills on the next `OnGameFrame` along the priority pyramid (`first_blood` → `combo.N` → `streak.N` → `streak_shutdown` → `weapons.<name>` → `headshot_streak.N` → `headshot` → `kill` → `teamkill`).
- Broadcasts include human spectators and CSTV while excluding T/CT bots; `!sound` stores only non-default overrides atomically in `player_preferences.json`.
- Exports `PlayCue`, `BroadcastCue`, `BroadcastRoundStartCue`, `PlaySpatialCue`, `StopCue`, and `HasTacticalCue`, tracking actual emitted event names for exact `StopSound` cancellation and enforcing valid entity handles on spatial cues.
