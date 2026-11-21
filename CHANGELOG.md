# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-11-21

### Added
- **多维度战斗与回合事件播报**：支持 `first_blood`、`combo.<N>`（基于 `combo_window_seconds` 的限时连杀）、`streak.<N>`、`streak_shutdown`（可配置 `shutdown_min_streak`）、`weapons.<name>` 动态武器映射、`headshot_streak.<N>`（回合内连续爆头，未命中具体次数时回退通用 `headshot`）、`kill`、`teamkill`、`suicide`、`round_start`、`freeze_end`、`round_end`（含 `round_end.ct` / `t` / `draw` 分结果路由）、`mvp` 与 `join_server`。
- **播放策略与击杀互斥开关**：`order` 支持 `sequential`（全服同图按序）、`actor_sequential`（按玩家槽位或发声实体独立按序）与 `random`；新增运行时 ConVar `imomoe_sounds_exclusive`（默认 `1`），支持同 Tick 多候选按优先级互斥单播或按维度叠加播报，且两种模式下连杀与爆头计数均正常推进。
- **JSONC 配置与 Schema 校验**：支持带注释与尾随逗号的 `configs/imomoe_sounds/sounds.jsonc`，配套提供 [sounds.schema.json](configs/imomoe_sounds/schemas/sounds.schema.json) 与 CS2 原生 / Workshop 示例配置。
- **Workshop 公共分发与预缓存**：通过 S2SDK `s2_extra_addons` 公共分发自定义音效包，在构建地图 Manifest 时逐图通过 `ReadFileVPK` 校验并全量 `Precache`。
- **玩家 `!sound` 偏好与聊天前缀集成**：支持中英文客户端自适应提示、基于 SteamID64 的增量原子写盘持久化，全服广播兼容观察者与 CSTV；可选集成 `imomoe_chat_prefix`（通过 `chat_prefix_marker` 注入前缀）。
- **Plugify 跨插件 ABI**：导出 `PlayCue`、`BroadcastCue`、`BroadcastRoundStartCue`、`PlaySpatialCue`、`StopCue` 与 `HasTacticalCue`，支持 3D 空间战术提示（按 `distance` 限距且不受 `!sound` 静音影响）与可中断模式 BGM。
