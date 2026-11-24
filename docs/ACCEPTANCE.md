# Imomoe Sounds 验收测试指南 / Acceptance Guide

[简体中文](#简体中文) | [English](#english)

---

## 简体中文

在隔离测试服上使用配对的 `imomoe_sounds`、FunRounds 与 S2SDK（`>=3.4.3`）执行以下正常路径验收。

---

### 1. 播报与个人偏好

顺序同步与个人静音隔离可用双人（A、B）配合 CT BOT 验收（BOT 可提供击杀事件，不作为收听端）：

1. 基于 [默认示例配置](../configs/imomoe_sounds/sounds.example.jsonc) 部署 `sounds.jsonc`。将 `announcements.kill.audience` 设为 `"all"`，A 用 AK 身体击杀两次：A、B 每次听到同一条提示且两次按配置顺序轮换；换图后重置回第一条。把 `announcements.kill.order` 改为 `"actor_sequential"` 并重启载入，让 A、B 各用 AK 身体击杀一次（都播第一条），再只让 A 被 BOT 击杀。A 重生后下一次击杀仍播第一条，B 下一次击杀播第二条；断线重连也会重置该玩家的轮播。改为 `"random"` 后允许连续重复。
2. 将 `announcements.kill.clips` 设为单条固定音轨并关闭 `announcements.streak`，击杀 CT BOT 默认可听；输入 `!sound` 收到关闭回显后再击杀只剩枪声。
3. 保持静音状态下依次重连、换图与正常重启服务端，击杀均应保持静音；再次输入 `!sound` 恢复播报后，`player_preferences.json` 不再保留该玩家的覆盖记录。双人测试时仅让 B 静音，A 击杀第三次应仅 A 听到播报。
4. 触发 `BroadcastCue(cue, 2)` 并在 T → CT → T 间切换阵营：T 听到提示、CT 无声、切回 T 再次听到。让 B 切至观察者验证全服播报；开启 CSTV 录制与回放，确认全服广播与可中断 BGM 正常记录。
5. 使用中英文客户端分别执行 `!sound`，核对本地化提示语；使用 `mixgroup = "UI"` 的工坊音轨时，将客户端 `snd_musicvolume` 设为 `0` 仍应听到播报。

---

### 2. 战斗优先级与边界

配置可区分的 `first_blood`、`combo.2`（`combo_window_seconds: 2.0`）、`streak.3`、`streak_shutdown`、`weapons.taser`、`headshot_streak.1` / `2`、`headshot`、`teamkill`、`suicide` 及 `freeze_end` / 回合首尾 / MVP 提示：

- `imomoe_sounds_exclusive` 为 `1`（默认）时，单次或同 Tick 合并击杀仅播报优先级最高的一条（`first_blood` → `combo.N` → `streak.N` → `streak_shutdown` → `weapons.<name>` → `headshot_streak.N` → `headshot` → `kill`）；设为 `0` 时按维度叠加播报，且同 Tick 同维度至多一条；两种模式下全部连杀与爆头计数照常推进。
- `first_blood` 每回合至多一次；`combo.N` 需在 `combo_window_seconds` 时间窗内连续击杀才触发（跨回合或阵亡重置）；`headshot_streak.N` 在回合内连续爆头时递进，未配置对应次数则回退通用 `headshot` 池；击杀 ≥3 连杀者时触发 `streak_shutdown`，被终结者下次击杀从连杀 1 起算；半场换边不清除连杀计数。
- 同 Tick 内用 HE 击杀多个目标时至多播报一声，后续击杀仍计入连杀数。
- 热身期正常播报且不污染正式局连杀与第一滴血；正式局首杀从第一滴血与连杀 1 重新开始。
- 同 Tick 先误杀队友再击杀敌人时，敌人击杀仍正常触发并可拿到第一滴血；已埋 C4 爆炸致死不触发 `suicide`。
- 非 FFA 下队友误杀走 `teamkill`，开启 FFA（`mp_teammates_are_enemies 1`）后按敌对击杀处理；自杀与世界伤害不消耗第一滴血。

---

### 3. 公共 Workshop 分发

参考 [Workshop 指南](WORKSHOP.md) 在 `s2_extra_addons` 添加音效包并部署 [Workshop 示例配置](../configs/imomoe_sounds/sounds.workshop.example.jsonc)：

- 使用未缓存该物品的客户端进服，确认自动触发下载并重连，并在不同地图（如 Dust II、Mirage）击杀均能正常播放自定义提示。
- 验证缺失资源时仅跳过受影响音轨并输出日志警告，不阻塞其他正常音轨。

---

### 4. FunRounds 联动与前缀检查

| 用例 | 预期结果 |
| --- | --- |
| 模式配置 `round_bgm_cue`，A 开启播报、B 静音 | 仅 A 听到模式开局 BGM；下一回合 `round_prestart` 自动停止未播完的 `interruptible` 音轨。 |
| 模式 BGM 与 `announcements.round_start` 同 Tick | 模式 BGM 发声成功时跳过默认开局播报；模式 Cue 不可用时回退播放默认开局播报。 |
| 幽灵呼吸（B 保持 `!sound` 关闭） | 存活玩家按本人位置限距，阵亡或观察者仅在第一人称跟随发声目标时按目标位置限距；超距无声，幽灵死亡或离开模式后立即停止。 |
| T 队启动自爆（`!sound` 关闭） | 正常倒计时先响警告，爆炸时停止警告并播放爆炸声；倒计时内被提前击杀时警告立即停止，且之后不会出现延迟爆炸声。 |
| 卸载/缺少 `imomoe_sounds` 启动 | 换图与退出无报错或悬挂句柄；无音效插件时 FunRounds 幽灵、自爆等玩法正常运行。 |
| 真人进服（`actor` / `all`）与 BOT 加入 | 仅按受众向未静音的真人播报；BOT 加入不触发 `join_server`。 |
| 配置 `round_end.ct` / `t` / `draw` 与公共 `round_end` | 各胜负结果播放对应列表；未配置结果键使用公共列表；设为 `clips: []` 的结果保持静音。 |
| 安装与卸载 `imomoe_chat_prefix` | 已安装时 `!sound` 聊天回复带彩色前缀，卸载或缺失时仅显示正文；控制台回复始终无标记与控制色。 |

---

## English

Run these happy-path acceptance checks on an isolated test server with paired `imomoe_sounds`, FunRounds, and S2SDK (`>=3.4.3`).

---

### 1. Announcements & Personal Preferences

Verify sequential sync and mute isolation using two human clients (A, B) plus CT bots (bots generate kill events, not listener state):

1. Deploy `sounds.jsonc` from [sounds.example.jsonc](../configs/imomoe_sounds/sounds.example.jsonc). Set `announcements.kill.audience` to `"all"` and have A score two AK body kills: A and B hear the same clip each time in order, resetting on map change. Set `announcements.kill.order` to `"actor_sequential"` and restart to load the config. Have A and B score one AK body kill each (both play clip 1), then have a bot kill A only. After A respawns, A's next kill plays clip 1 again and B's next kill plays clip 2. Reconnecting also resets that player's rotation. Switch to `"random"`: consecutive repeats are allowed.
2. Set `announcements.kill.clips` to a single clip and disable `announcements.streak`. Kill a CT bot (audible), run `!sound` to mute, and kill again (gunshot only).
3. Keep muted across reconnect, map change, and clean server restart; toggle `!sound` back on and confirm `player_preferences.json` removes that player's override. With only B muted, A's third kill is heard only by A.
4. Trigger `BroadcastCue(cue, 2)` while switching T → CT → T: heard on T, silent on CT, heard again on T. Move B to spectator to verify all-audience broadcasts; verify CSTV recording/playback captures world broadcasts and interruptible BGM.
5. Run `!sound` on English and Chinese clients to verify localization; with `mixgroup = "UI"` Workshop tracks, verify cues remain audible at `snd_musicvolume 0`.

---

### 2. Combat Priority & Edge Cases

Configure distinguishable cues for `first_blood`, `combo.2` (`combo_window_seconds: 2.0`), `streak.3`, `streak_shutdown`, `weapons.taser`, `headshot_streak.1` / `2`, `headshot`, `teamkill`, `suicide`, and `freeze_end` / round / MVP cues:

- With `imomoe_sounds_exclusive 1` (default), emit only the highest-priority cue per tick (`first_blood` → `combo.N` → `streak.N` → `streak_shutdown` → `weapons.<name>` → `headshot_streak.N` → `headshot` → `kill`); with `0`, emit up to one cue per dimension per tick; verify all counters advance in both modes.
- Verify `first_blood` fires at most once per round, `combo.N` requires consecutive kills within `combo_window_seconds` (resets on death/round), `headshot_streak.N` increments on consecutive headshots (falling back to `headshot`), `streak_shutdown` triggers on killing ≥3-streak players, and half-time side swap preserves streaks.
- Multi-kill HE grenade in one tick emits once while incrementing streaks for all kills.
- Warmup announces normally without polluting live-match first blood or streaks.
- Teamkill + enemy kill in the same tick still grants the enemy kill and first blood; planted C4 deaths do not trigger `suicide`.
- Non-FFA friendly kills route to `teamkill`; FFA (`mp_teammates_are_enemies 1`) treats them as hostile kills; suicides and world damage do not consume first blood.

---

### 3. Public Workshop Distribution

Follow [Workshop Guide](WORKSHOP.md#english) to add the addon ID to `s2_extra_addons` and deploy [sounds.workshop.example.jsonc](../configs/imomoe_sounds/sounds.workshop.example.jsonc):

- Join with an uncached client to verify auto-download, reconnect, and playback across maps (e.g. Dust II, Mirage).
- Verify missing resources log warnings and skip only affected tracks without blocking valid cues.

---

### 4. FunRounds Integration & Prefix Checks

| Case | Expected Result |
| --- | --- |
| Mode sets `round_bgm_cue`; A unmuted, B muted | Only A hears mode BGM; `round_prestart` stops unfinished `interruptible` tracks. |
| Mode BGM & `announcements.round_start` same tick | Emitted mode BGM suppresses default `round_start`; unavailable mode cue falls back to default `round_start`. |
| Ghost breathing (B muted via `!sound`) | Alive players filter by own position; dead/spectators filter only in first-person spectating target; stops immediately on ghost death or mode end. |
| T suicide bomber (`!sound` off) | Countdown warning plays; explosion stops warning and plays boom; early kill stops warning immediately with no delayed boom. |
| Unload / run without `imomoe_sounds` | Map change and shutdown produce no errors or dangling handles; FunRounds gameplay runs normally without Sounds. |
| Human `join_server` (`actor` / `all`) vs BOT join | Announces only to unmuted humans per audience; bots never trigger `join_server`. |
| `round_end.ct` / `t` / `draw` vs shared `round_end` | Matches winner/draw routes; missing keys fall back to `round_end`; `clips: []` stays silent. |
| `imomoe_chat_prefix` loaded vs absent | `!sound` chat reply prepends colored prefix when loaded and plain body when absent; console reply stays plain text. |
