# FunRounds 联动配置说明

本文档说明 `imomoe_sounds` 与 `funrounds` 的配置对接契约。游戏规则与触发时机由 FunRounds 控制，`imomoe_sounds` 仅按当前配置选曲与发声，不会因曲目缺失或未加载音效插件而阻断玩法。

---

## FunRounds 玩法侧联动与前缀配置

- **模式开局 BGM**：在 FunRounds 模式配置中声明 `round_bgm_cue = "funrounds.flavor.<名称>"`（可在 `[defaults]` 统一设置，并在个别模式下设为 `false` 清除）。该 Cue 须对应非空间音轨；当 `BroadcastRoundStartCue` 在同 Tick 成功发出模式 BGM 时，Sounds 会跳过默认的 `announcements.round_start` 播报；若模式 Cue 不可用或未发出则保留默认播报。
- **幽灵呼吸与空间距**：FunRounds 负责配置 `breath_interval`（若需关闭呼吸请将 `breath_interval` 设为 `0.0`）与 `breath_distance` 等节拍与距离上限；实际发声距离上限取玩法侧 `distance_cap`（非零时）与 `sounds.jsonc` 中 `distance` 的较小值。
- **聊天前缀**：配置 `chat_prefix_marker`（如 `"<prefix>"` 或 `"{prefix}"`）后，Sounds 会在 `!sound` 聊天回复前拼接标记；若安装 `imomoe_chat_prefix` 则替换为实际前缀，缺失时自动清理标记与相邻空格。控制台回复始终不输出标记及控制色。

---

## Cue 配置映射

`clips` 可混用 CS2 内置事件与在 `workshop.resources` 中登记的自定义事件，支持 `sequential` / `actor_sequential` / `random` 与空列表静音：

```jsonc
"chrono_teleport": {
  "spatial": true,
  "order": "sequential",
  "clips": ["UI.CoinLevelUp", "Custom.Teleport"]
}
```

### FunRounds 战术空间键（`funrounds.tactical.*`）

所有战术键的完整调用名以 `funrounds.tactical.` 开头，必须配置 `"spatial": true`，受众固定为 `all` 且不受 `!sound` 个人静音控制：

| 战术键 | 触发场景 |
| --- | --- |
| `ghost_breath` | 幽灵呼吸 |
| `suicide_warning`、`suicide_explosion` | 自爆倒计时警告、自爆爆炸 |
| `chrono_teleport` | Zeus 空间传送落地 |
| `chrono_stasis_lock`、`chrono_stasis_kill` | 引导抹消锁定、成功击杀 |
| `grapple_attach`、`grapple_release` | 抓钩挂接、释放 |
| `explosive_bullet` | 爆炸子弹爆炸 |
| `chicken_eat_heal`、`chicken_eat_overfull`、`chicken_eat_fatal`、`chicken_eat_switch` | 吃鸡回血、吃撑受伤、吃撑致死、触发自爆开关 |
| `chicken_explosion`、`chicken_revenge` | 杀鸡爆炸、普通鸡复仇反伤 |
| `c4_revenge_spawn`、`c4_revenge_explosion`、`c4_revenge_retire` | C4 复仇鸡出场、爆炸、击杀后隐退 |
| `chicken_idle`、`chicken_panic` | 变鸡待机、受惊 |

### 受 `!sound` 控制的联动与播报键

| 配置键 | 触发与受众说明 |
| --- | --- |
| `announcements.combo.<N>` | `combo_window_seconds`（默认 `2.0`，`0` 关闭）时间窗内连续敌对击杀；支持 `audience` 为 `all` 或 `actor` |
| `announcements.headshot_streak.<N>` | 回合内连续爆头（阵亡/换回合/断线重置）；未命中具体次数时回退通用 `headshot` 池 |
| `announcements.join_server` | 真人完成连接（跳过 BOT）；`audience` 可选 `actor` 或 `all` |
| `announcements.round_end.ct`、`round_end.t`、`round_end.draw` | CT 胜、T 胜或无胜方且 reason 为 Draw（10）；受众固定为 `all` |
| `announcements.round_end` | 未配置对应结果键时的公共回退；结果键设为空列表时静音 |
| `funrounds.flavor.player_blind` | 被致盲玩家；必须为非空间音轨 |
| `funrounds.flavor.regeneration` | 受到回血效果的玩家；必须为非空间音轨 |
| `funrounds.flavor.<名称>` | 由模式 `round_bgm_cue` 指定的开局 BGM |

> [!TIP]
> 战斗播报可通过 ConVar `imomoe_sounds_exclusive`（默认 `1`）控制互斥策略：`1` 时单次或同 Tick 合并击杀只播优先级最高的一条，`0` 时按维度叠加播报，且连杀与爆头计数在两种模式下均正常推进。

> [!NOTE]
> `round_end.*` 结果路由依赖 S2SDK 的 `GetEventInt` 导出（见 [bindings/s2sdk.pplugin](../bindings/s2sdk.pplugin)）。若运行在缺少该导出的裁剪版 provider 上，启动日志会记录警告并回退至公共 `round_end`。
