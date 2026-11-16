# Imomoe Sounds

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Rust: 1.99.1](https://img.shields.io/badge/Rust-1.99.1%20(2024)-orange.svg)](rust-toolchain.toml)
[![CS2: Compatible](https://img.shields.io/badge/CS2-Dedicated%20Server-black.svg)](https://www.counter-strike.net/)
[![Platform: Plugify](https://img.shields.io/badge/Plugify-v3.1.0%2B-purple.svg)](https://github.com/untrustedmodders/plugify)
[![CI](https://github.com/Imomoe-Community/Imomoe-Sounds/actions/workflows/ci.yml/badge.svg)](https://github.com/Imomoe-Community/Imomoe-Sounds/actions/workflows/ci.yml)

面向 CS2 Dedicated Server 的 Plugify 音效插件。负责战斗与回合事件播报、Workshop 自定义声效资源预缓存、空间战术提示以及玩家个人播报偏好开关（`!sound`）；玩法模组（如 FunRounds）只需按需调用导出接口触发对应 Cue。

---

## 环境要求

- CS2 Dedicated Server 与兼容的 Plugify（`v3.1.0+`）
- Plugify Rust Module（`1.3.0`，若使用 `1.2.5` 请注意 Linux 产物的文件名约定）
- S2SDK `>=3.4.2`（若使用裁剪过的 S2SDK provider，请确保导出清单包含 [bindings/s2sdk.pplugin](bindings/s2sdk.pplugin) 声明的所需方法与原型）
- （可选）[Imomoe Chat Prefix](https://github.com/Imomoe-Community/Imomoe-Chat-Prefix)（`imomoe_chat_prefix >=0.1.0`），用于统一 `!sound` 聊天回复前缀

---

## 安装与部署

1. 构建发布产物：

   ```bash
   cargo build --release --locked
   ```

2. 将 [imomoe_sounds.pplugin](imomoe_sounds.pplugin) 与编译产物放入 `game/csgo/addons/plugify/envs/imomoe_sounds/`：
   - **Windows**：将 `target/release/imomoe_sounds.dll` 放入 `bin/imomoe_sounds.dll`。
   - **Linux**：将 `target/release/libimomoe_sounds.so` 放入 `bin/libimomoe_sounds.so`。

3. 创建运行时配置目录 `game/csgo/addons/plugify/configs/imomoe_sounds/`：
   - 将 [schemas/sounds.schema.json](configs/imomoe_sounds/schemas/sounds.schema.json) 复制到该目录的 `schemas/` 下（用于编辑器校验与自动补全）。
   - 将示例配置 [sounds.example.jsonc](configs/imomoe_sounds/sounds.example.jsonc)（原生 CS2 音效预设，无需 Workshop）或 [sounds.workshop.example.jsonc](configs/imomoe_sounds/sounds.workshop.example.jsonc)（Workshop 自定义音效预设）复制为 `sounds.jsonc` 并按需修改。

4. 重启服务端或按运维流程重新加载插件使其生效。

> [!TIP]
> **从 0.1.0 升级**：请将旧 `sounds.toml` 列表迁入 `sounds.jsonc`（常规列表放入 `announcements`，`streak.N` 放入 `announcements.streak.N`，武器放入 `weapons.<事件武器名>`）。Workshop 自定义资产现改用 `s2_extra_addons` 公共分发，不再使用私有分发名单或逐玩家原生回退表。详细迁移说明见 [FunRounds 联动说明](docs/FUNROUNDS_INTEGRATION.md)。

---

## 配置播放与收听

配置文件支持 JSONC 注释与尾随逗号，并通过 [sounds.schema.json](configs/imomoe_sounds/schemas/sounds.schema.json) 进行严格校验（不允许未知字段）：

- **播放策略**：`order` 支持 `sequential`（同图按序循环，换图复位）与 `random`（等概率独立抽取）。`clips` 可混填 CS2 内置事件与已登记的自定义 SoundEvent；省略配置项将跳过该候选，显式配置 `clips: []` 则静音。
- **`announcements`**：配置回合、MVP 与战斗播报。击杀事件按优先级只播放一个 Cue：`first_blood` → `streak.N` / `streak_shutdown` → `weapons.<name>` → `headshot` → `kill`；非 FFA 误杀（`teamkill`）与自杀（`suicide`）独立触发。同一击杀者同一 Tick 多次敌对击杀不叠播，但仍计入连杀。
- **`announcements.join_server` 与 `round_end.*`**：`join_server` 可设置 `audience` 为 `"actor"` 或 `"all"`；回合结算支持 `round_end.ct`、`round_end.t` 与 `round_end.draw`（仅无胜方且结算原因为 Draw 时触发），省略结果键时回退至公共 `round_end`，设 `clips: []` 则该结果静音。
- **`funrounds.flavor`**：模式 BGM 等装饰音；支持 `"interruptible": true`，在下一回合 `round_prestart` 时自动停止。
- **`funrounds.tactical`**：实体空间战术提示（如幽灵呼吸、自爆警告与爆炸），按 `distance` 筛选且不受 `!sound` 个人静音影响；空间衰减由 SoundEvent 自身定义负责。
- **`workshop`**：仅使用自定义音效时需要。填写 `id`、`events`（`.vsndevts`）与 `resources`（`.vsnd`），并将物品 ID 加入服务端 `s2_extra_addons`。详见 [Workshop 指南](docs/WORKSHOP.md)。

### 玩家开关与聊天前缀

- 玩家在聊天或控制台输入 `!sound` 切换个人播报开关。偏好以 SteamID64 为键即时原子写入 `player_preferences.json`，仅保存与服务端 `enabled_by_default` 不同的设置。
- 配置 `chat_prefix_marker`（如 `"<prefix>"`）且安装 `imomoe_chat_prefix` 时，`!sound` 聊天回复会自动注入当前社区前缀；前缀插件缺失时自动移除标记与分隔空格，控制台回复始终保持纯文本。

---

## Plugify API

在游戏主线程传入配置中的 Cue 标识（例如 `kill`、`funrounds.flavor.knife_round_start` 或 `funrounds.tactical.ghost_breath`）：

| 方法 | 说明 |
| --- | --- |
| `PlayCue(cue, slot)` | 向指定玩家槽位播放 Cue，尊重个人 `!sound` 设置。 |
| `BroadcastCue(cue, team)` | 向 `0` 全服（含观察者与 CSTV）、`2` T 或 `3` CT 播报，尊重真人玩家设置。 |
| `BroadcastRoundStartCue(cue)` | 发出模式开局音轨；发声成功时替代同 Tick 的默认 `round_start` 播报。 |
| `PlaySpatialCue(cue, entity_handle, distance_cap, tactical)` | 从有效完整实体句柄发声。`distance_cap` 为 `0` 时不加调用侧上限，正数则与配置 `distance` 取较小值；`tactical` 必须与 Cue 分类匹配。 |
| `StopCue(cue, entity_handle)` | 停止该实体上此 Cue 最近一次实际播放的音轨。 |
| `HasTacticalCue(cue)` | 查询战术 Cue 当前是否至少有一条可用或已预缓存的音轨；不应用于阻断玩法逻辑。 |

对播放与停止方法，返回 `true` 表示已向引擎发出请求；无匹配 Cue、无合格听众或实体句柄无效时返回 `false`（不会回退到世界实体）。

---

## 构建与开发

本仓库使用 Rust 2024 Edition，工具链固定为 `1.99.1`（见 [rust-toolchain.toml](rust-toolchain.toml)）。提交改动前运行标准四门禁：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

更新 [bindings/s2sdk.pplugin](bindings/s2sdk.pplugin) 后重新生成绑定：

```bash
plugify-gen -lang rust -manifest bindings/s2sdk.pplugin -output . -overwrite
cargo fmt
```

### 相关文档

- [架构说明](docs/ARCHITECTURE.md) · [Workshop 指南](docs/WORKSHOP.md) · [FunRounds 联动说明](docs/FUNROUNDS_INTEGRATION.md)
- [验收测试指南](docs/ACCEPTANCE.md) · [联动设计 RFC](docs/RFC_FUNROUNDS_INTEGRATION.md)
- [贡献指南](CONTRIBUTING.md) · [安全通告](SECURITY.md) · [版本记录](CHANGELOG.md)

---

## 许可证

[GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later)](LICENSE)。
