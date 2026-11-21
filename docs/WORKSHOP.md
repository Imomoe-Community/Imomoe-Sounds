# Workshop 自定义音效配置与分发指南

Imomoe Sounds 通过 S2SDK 的 `s2_extra_addons` 将同一份 Workshop 音效包公共分发给所有客户端。玩家首次进服时自动下载并在本地缓存，后续复用 Steam 缓存。仅使用 [默认示例预设](../configs/imomoe_sounds/sounds.example.jsonc) 中的 CS2 内置音效时，无需配置 Workshop。

---

## 1. 接入已有 Workshop 音效包

1. 将音效包物品 ID 加入服务端 `s2_extra_addons`：

   ```text
   s2_extra_addons "<WORKSHOP_ID>"
   ```

   - **多个物品 ID**：以英文半角逗号 `,` 分隔，**逗号前后不要加空格**（例如 `s2_extra_addons "3763619947,1234567890"`）。
   - **保留 S2SDK 默认图标包**：S2SDK 的 `s2_extra_addons` 默认包含 `"3763619947"`（用于内置 WASD 按钮菜单图标）；若服务端用到了该菜单并覆写了 `s2_extra_addons`，记得将 `3763619947` 一并保留。

2. 复制 [Workshop 示例配置](../configs/imomoe_sounds/sounds.workshop.example.jsonc) 为服务端 `configs/imomoe_sounds/sounds.jsonc`（保留同级 `schemas/` 目录），将 `workshop.id` 改为实际物品 ID，并在 `workshop.resources` 中为每个自定义 SoundEvent 登记一次 `.vsnd` 路径。
3. 启动或切换地图。插件在 `OnBuildGameSessionManifest` 阶段验证物品是否已挂载，读取编译后的 `.vsndevts_c` 与 `.vsnd_c` 并自动预缓存可用资源；缺失的资源会输出警告日志并跳过对应曲目。

---

## 2. 服务端自动更新与存放位置

- **自动检查更新**：S2SDK 的 `s2_addon_mount_download` 默认值为 `0`（已安装过的 Workshop 物品不会主动拉取更新）。如需服务端自动跟进音效包更新，建议在服务端配置加入：

  ```text
  s2_addon_mount_download 1
  ```

  开启后每次启动或换图挂载时会在后台拉取更新（不中断当前对局），并在**下一次换图或重启**时挂载新版 VPK；若保持 `0`，可在更新工坊后执行 `s2_download_addon <WORKSHOP_ID>`，待日志提示下载完成后换图或重启生效。
- **VPK 存放路径**：专用服将 Workshop 内容下载到可执行文件目录（`EXECUTABLE_PATH`）下的 `steamapps/workshop/content/730/<WORKSHOP_ID>/`（例如 `game/bin/win64/steamapps/workshop/content/730/<WORKSHOP_ID>/` 或 `game/bin/linuxsteamrt64/steamapps/workshop/content/730/<WORKSHOP_ID>/`）。运行期可通过服务端控制台 `s2_print_searchpaths` 核对当前挂载的 VPK 路径。

---

## 3. 制作与编译 SoundEvent 规范

- **UI 播报与背景音乐（`announcements` / `funrounds.flavor`）**：
  - 在 Source 2 `.vsndevts` 中将 `mixgroup` 设为 `"UI"`，并关闭距离衰减与混响（`use_distance_volume_mapping_curve = false`、`distance_effect_mix = 0.0`），以确保不被客户端 `snd_musicvolume 0` 屏蔽。
- **空间战术提示（`funrounds.tactical`）**：
  - 建议使用单声道（Mono）音频源，并在 `.vsndevts` 中配置合适的 3D 空间衰减曲线；不要直接套用 2D UI 播报模板。
- **响度与编译**：
  - 建议在制作阶段统一归一化音频响度，并通过 CS2 Workshop Tools 资产浏览器编译生成 `.vsnd_c` 与 `.vsndevts_c` 后再发布或更新物品。

---

## 4. 客户端下载验收

使用未本地缓存该 Workshop 物品的客户端进行 [公共分发验收](ACCEPTANCE.md#3-公共-workshop-分发)。若复用测试客户端，请先退出 CS2，在 Steam 客户端中订阅并等待下载完成后再取消订阅，以同时清理本地文件与 Steam 安装记录。
