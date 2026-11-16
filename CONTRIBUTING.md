# Contributing to Imomoe Sounds

感谢对 Imomoe Sounds 项目的关注与贡献。在提交代码或 Pull Request 之前，请阅读以下开发与协同规范。

---

## 开发环境与工具链

本项目基于 Rust 2024 Edition，绑定并依赖 [Plugify](https://github.com/untrustedmodders/plugify) 与 [S2SDK](https://github.com/untrustedmodders/plugify-plugin-s2sdk)。

- **Rust 工具链**：请确保通过 `rust-toolchain.toml` 使用固定的 `1.99.1` 版本，包含 `rustfmt` 与 `clippy` 组件：
  ```bash
  rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy
  ```
- **操作系统支持**：
  - Windows: x86_64 MSVC
  - Linux: x86_64 GNU（发布环境建议基于 Steam Runtime Sniper）

---

## 质量门禁与本地验证

所有提交必须通过以下四项标准门禁：

```bash
# 1. 格式化检查
cargo fmt --all -- --check

# 2. 静态检查（严格禁止告警）
cargo clippy --all-targets --locked -- -D warnings

# 3. 单元测试
cargo test --locked

# 4. 发布构建
cargo build --release --locked
```

---

## 核心设计与编码规范

1. **FFI 边界安全**：
   - 暴露给游戏引擎或 Plugify 的外部 C ABI 入口（如 `start`、`end`、`PlayCue` 及各类事件回调）必须使用 `catch_unwind` / `boundary` 宏拦截 panic，严禁 panic 跨越 C ABI 导致进程崩溃。
   - 游戏引擎调用必须在游戏主线程安全执行。

2. **状态与生命周期契约**：
   - 玩家断线清理该槽位的连杀和防抖记录；个人偏好按 SteamID64 保留。实体删除清理对应空间音轨。
   - 换图（`on_map_end` / `on_map_start`）清理运行时状态并保证播放列表按序正确复位。
   - 插件卸载（`end`）必须精准撤销全部已注册的 Console Command、事件 Hook 及 Manifest 监听器，确保零内存泄漏与悬挂回调。

3. **配置与分发契约**：
   - 配置解析采用严苛模型（`deny_unknown_fields`），杜绝因配置拼写错误导致的隐蔽异常。
   - 公共 Workshop 资源在构建地图清单时检查并预缓存。未就绪的自定义曲目不参与选曲；不维护每客户端的原生回退表。

4. **S2SDK 绑定维护**：
   - 如需更新 SDK 接口绑定，使用 `bindings/s2sdk.pplugin` 运行生成器：
     ```bash
     plugify-gen -lang rust -manifest bindings/s2sdk.pplugin -output . -overwrite
     cargo fmt
     ```

---

## 提交与 Pull Request 流程

1. 基于 `main` 分支拉取新的特性分支（如 `feat/my-feature` 或 `fix/my-bug`）。
2. 保持每次提交聚焦单一主题，提交说明简练准确；切勿提交个人运行配置（`sounds.jsonc`、`player_preferences.json`）、真实社区 Workshop ID 或服务端私有信息。
3. 确保本地四门禁全部通过后发起 Pull Request。
4. PR 描述清晰说明改动动机、影响范围及验证方法（必要时提供游戏内实机测试记录）。
