# RFC: FunRounds 深度集成与多维音效系统架构规划

> [!NOTE]
> 本文档为早期设计记录。后续演进已新增 `BroadcastRoundStartCue` 与 `HasTacticalCue` 导出方法，并移除了私有分发回退表与旧打包脚本；当前完整接口与实际配置请以 [README](../README.md#plugify-api) 及 [FunRounds 联动配置说明](FUNROUNDS_INTEGRATION.md) 为准。

本文档记录了 `imomoe_sounds` 音效插件与 `funrounds` 玩法模组深度解耦、接口补齐、全服音效统一治理及最终确认的工程实现细节。

---

## 1. 核心设计原则与已确认路线

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

## 2. 事件系统扩展与战斗逻辑

常规战斗事件在 CS2 引擎中原生具备完整上下文，由音效插件直接 Hook 捕获并处理：

- **回合节奏事件**：
  - `round_prestart`：强制中断并冲刷上一回合未播完的 `interruptible` 长 BGM 与氛围音。
  - `round_start` / `round_freeze_end`（冻结结束战斗打响提示音）/ `round_end`。
- **第一滴血 (First Blood)**：回合重置后首次产生非自杀敌对击杀时触发，全服广播。
- **连杀与终结 (Streak & Shutdown)**：支持多级连杀播报，玩家自身阵亡时连杀计数归零；**半场换边时保留连杀计数**；击杀高连杀者时触发终结连杀（`streak_shutdown`）播报。
- **团队误杀与自杀 (Teamkill & Suicide)**：实时读取 `mp_teammates_are_enemies` 判定 FFA 模式（开启时全员视为敌人）；非 FFA 下同阵营误杀触发 `teamkill`，坠亡/自爆/世界伤害触发 `suicide`。
- **武器动态映射 (Dynamic Weapon Cues)**：直接读取 `player_death` 中的 `weapon` 字符串动态检索 `weapons.<name>`（如 `taser`、`knife`、`hegrenade`、`inferno` 等），零代码改动扩展任意武器。

### 优先级裁决与同 Tick 防抖（方案 A）
- **同 Tick 同步防抖（零延迟）**：
  - 内部维护每个凶手最后一次发声的 `last_kill_tick`（基于 `s2sdk::GetGameTickCount()`）。
  - 同一服务器 Tick 内因手雷/自爆造成的瞬间多次击杀，第 2 次及后续击杀正常累计 `streak` 连杀数，但**跳过重复发声**，防止瞬间叠加爆音，保持零延迟响应。
- **优先级金字塔**：
  $$\text{First Blood (若未触发)} \longrightarrow \text{连杀 / 终结连杀} \longrightarrow \text{特定武器} \longrightarrow \text{爆头} \longrightarrow \text{普通击杀} \longrightarrow \text{TK / 自杀}$$

---

## 3. 客户端指令、CSTV 兼容与持久化设计

1. **Mixgroup 与个人播报开关**：
   - 所有非空间提示音与氛围 BGM 在工坊 KV3 中统一归入 `mixgroup = "UI"`，确保不被玩家客户端的 `snd_musicvolume 0` 屏蔽。
   - 提供 `!sound` 开关播报。R3 实机未听出不同非零百分比的差异，因此取消个人和配置内的音量调节，按声音事件本身播放。
   - **语言自适应**：调用 `s2sdk::GetClientLanguageCode(slot)` 依据玩家客户端语言自动返回中文或英文反馈。
2. **观察者与 CSTV 精确筛选**：
   - 全局广播收集目标槽位时：
     - 普通真人玩家：`IsClientInGame(slot) && !IsFakeClient(slot)`（尊重个人 `!sound` 设置）。
     - CSTV 端：`IsClientInGame(slot) && IsClientSourceTV(slot)`，用 SDK 的专门身份查询识别不一定拥有 pawn 的 CSTV 录制端，同时跳过 T/CT 阵营的普通 AI BOT。
3. **零冗余偏好持久化**：
   - 采用轻量级本地 JSON（`configs/imomoe_sounds/player_preferences.json`），以 SteamID64 为键。
   - **仅存储非默认设置**（采用服务端默认开关的玩家不写盘），避免过客玩家撑爆配置文件。
   - 玩家切换开关时在同目录写临时文件并原子替换；失败时保留本次会话的设置，在换图或正常退出时重试尚未写入的更改。

---

## 4. 对外导出接口与工程契约 (Plugify ABI)

音效插件为玩法模组提供以下四个正交的基础 ABI 原语（未配置的 `cue` 静默返回 `false`，不刷屏报错）：

```rust
/// 1. 单人 UI 播报接口 (受 !sound 控制)
pub extern "C" fn PlayCue(cue: &Str, slot: i32) -> bool;

/// 2. 全局 / 阵营广播接口 (team: 0 = 全服含观察者与 CSTV, 2 = T, 3 = CT)
pub extern "C" fn BroadcastCue(cue: &Str, team: i32) -> bool;

/// 3. 空间实体音效接口 (挂载在 Pawn 等实体，具备距离衰减与物理定位)
pub extern "C" fn PlaySpatialCue(
    cue: &Str,
    entity_handle: i32,
    distance_cap: f32,
    tactical: bool
) -> bool;

/// 4. 音效终止接口 (用于掐断幽灵呼吸或被提前击毙的自爆引信)
pub extern "C" fn StopCue(cue: &Str, entity_handle: i32) -> bool;
```

### 关键实现约束
1. **循环节拍归属**：幽灵呼吸的触发间隔、存活判定与倒计时等玩法逻辑保留在 FunRounds 中；`imomoe_sounds` 仅负责执行单次 `PlaySpatialCue` 与响应 `StopCue`。
2. **活跃音轨精确追踪（适配 `StopSound`）**：
   - S2SDK 3.4.2 的 `StopSound(entityHandle, sound)` 要求传入精确的音频事件名，而播放列表采用随机/顺序多选池。
   - 插件内部维护 `active_entity_cues: HashMap<(EntityHandle, CueName), ActualClipName>` 与 `active_world_interruptible: Vec<ActualClipName>`。调用 `StopCue` 或触发 `round_prestart` 时，插件自动查表找到实际播放的 `ActualClipName` 并执行 `StopSound`。
3. **空间发声实体有效性校验**：
   - `PlaySpatialCue` 入口处强制检验 `s2sdk::IsValidEntHandle(entity_handle)`。
   - 若传入瞬间实体已销毁或失效（如玩家提前阵亡/断线、临时特效过期），**直接静默返回 `false`**，绝不强制降级为 `world (0)` 全图发声（避免“灵异空气音效”及无效坐标读取）。
   - 距离计算使用 `GetEntityAbsOrigin(entity_handle)` 获取声源坐标，与各听众的 `GetClientAbsOrigin(slot)` 对比过滤。

---

## 5. 配置文件演进方案：直接切换 JSONC 规范

项目处于 `0.1.0` 早期迭代阶段，直接将默认配置切换为支持注释的 `configs/imomoe_sounds/sounds.jsonc`（不保留旧 TOML 双格式历史包袱），配套提供 `sounds.schema.json` 以支持编辑器实时补全与静态校验：

```jsonc
{
  "$schema": "./schemas/sounds.schema.json",
  "enabled_by_default": true,

  // 1. 全局播报类 (受 !sound 控制)
  "announcements": {
    "first_blood": { "audience": "all", "order": "random", "clips": ["Imomoe.first_blood_1"] },
    "freeze_end": {
      "audience": "all",
      "order": "random",
      "interruptible": true, // round_prestart 时自动打断
      "clips": [
        "Imomoe.freeze_end_1",
        "Imomoe.freeze_end_2"
      ]
    },
    "kill": { "order": "sequential", "clips": ["Imomoe.kill_a", "Imomoe.kill_b"] },
    "headshot": { "clips": ["Imomoe.headshot"] },
    "streak": {
      "3": { "clips": ["Imomoe.streak_3"] },
      "5": { "clips": ["Imomoe.streak_5"] }
    },
    "streak_shutdown": { "audience": "all", "clips": ["Imomoe.shutdown"] },
    "teamkill": { "clips": ["Imomoe.teamkill"] },
    "suicide": { "clips": ["Imomoe.suicide"] }
  },

  // 2. 武器击杀特化 (动态武器名映射，受 !sound 控制)
  "weapons": {
    "taser": { "order": "random", "clips": ["Imomoe.taser_1", "Imomoe.taser_2"] },
    "knife": { "clips": ["Imomoe.knife"] },
    "hegrenade": { "clips": ["Imomoe.grenade"] },
    "inferno": { "clips": ["Imomoe.fire"] }
  },

  // 3. FunRounds 玩法专属分类 (物理路径区分，无需侵入式标记)
  "funrounds": {
    // 3.1 模式氛围类 (受 !sound 控制)
    "flavor": {
      "knife_round_start": { "audience": "all", "order": "random", "interruptible": true, "clips": ["Imomoe.bgm_knife"] },
      "zombie_infection": { "audience": "all", "interruptible": true, "clips": ["Imomoe.bgm_zombie"] }
    },
    // 3.2 战术机制类 (强制全员收听，支持实体空间发声与多样化音效池)
    "tactical": {
      "ghost_breath": {
        "spatial": true,
        "distance": 1000.0,
        "order": "random",
        "clips": ["Imomoe.breath_1", "Imomoe.breath_2"]
      },
      "suicide_warning": {
        "spatial": true,
        "distance": 1200.0,
        "order": "random",
        "clips": ["Imomoe.suicide_warn_1", "Imomoe.suicide_warn_2"]
      }
    }
  }
}
```

---

## 6. FunRounds 特殊回合配置联动契约与实施路线

- **纯文本松耦合引用**：FunRounds 回合模式配置仅声明对应 Cue 标识：
  ```toml
  [modes.knife]
  round_bgm_cue = "funrounds.flavor.knife_round_start"
  ```
- **轻量触发**：FunRounds 在阶段转换时直接调用 `BroadcastCue("funrounds.flavor.knife_round_start", 0)`。

### 分阶段实施路线
1. **阶段一（音效插件独立升级）**：
   - 更新 `bindings/s2sdk.pplugin` 补齐 `StopSound`、`GetEntityAbsOrigin`、`GetClientAbsOrigin`、`GetGameTickCount`、`IsValidEntHandle`、`GetClientLanguageCode` 绑定；
   - 实现 `sounds.jsonc` + `sounds.schema.json` 配置解析、战斗细分事件、同 Tick 防抖、`!sound` 指令、CSTV 过滤、JSON 持久化与 4 个导出 ABI。
2. **阶段二（FunRounds 联动接入）**：
   - 将 FunRounds 中的硬编码幽灵呼吸、自爆预警/爆炸声音切换至 `imomoe_sounds` 导出接口，并在模式配置中支持自定义 Cue 触发。
