# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-11-17

### Added
- 新增第一滴血（`first_blood`）、终结连杀（`streak_shutdown`）、团队误杀（`teamkill`）、自杀（`suicide`）、冻结结束（`freeze_end`）、玩家进服（`join_server`）、分阵营/平局回合结算（`round_end.ct` / `t` / `draw`）及动态武器映射（`weapons.<name>`）。
- 同一击杀者同一 Tick 多次敌对击杀自动合并并按优先级只播一声，同时完整计入连杀。
- `!sound` 切换支持中英文客户端自适应反馈与即时原子写盘；全服广播支持观察者与 CSTV 录制端。
- 导出 `BroadcastCue`、`BroadcastRoundStartCue`、`PlaySpatialCue`、`StopCue` 与 `HasTacticalCue` Plugify ABI，支持 FunRounds 战术空间音与可中断模式 BGM。
- 可选集成 `imomoe_chat_prefix`，通过 `chat_prefix_marker` 为 `!sound` 聊天回复添加统一社区前缀。

### Changed
- 配置文件由 `sounds.toml` 升级为带 JSON Schema 校验的 `sounds.jsonc`（示例见 `configs/imomoe_sounds/sounds.example.jsonc`），S2SDK 最低版本要求为 `>=3.4.2`。
- 自定义音效改为 `s2_extra_addons` 公共分发，单条音轨仅声明一次资源路径；不再维护每玩家原生回退映射与个人音量调节。

### Removed
- 清理开发期遗留的旧补丁文件、打包脚本与音效生成工具目录，仓库仅保留插件代码、SDK 绑定清单、Schema 与示例配置。

## [0.1.0] - 2026-11-12

### Added
- **多通道音效编排系统**：支持配置 `kill`、`headshot`、`knife`、`streak.N`、`mvp`、`round_start`、`round_end` 等多类游戏事件播报。
- **播放调度机制**：支持 `sequential`（按配置顺序循环）与 `random`（等概率独立抽取）播放策略与受众过滤（`actor` / `all`）。
- **个性化音效偏好**：提供 `!sound` 玩家指令，支持换图与断线重连期间的偏好记忆。
- **跨插件调用接口**：对外导出 Plugify 方法 `PlayCue(cue, slot) -> bool`。
