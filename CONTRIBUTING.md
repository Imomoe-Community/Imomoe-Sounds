# Contributing Guide / 贡献指南

感谢对 **Imomoe Sounds** 项目的关注与贡献！在提交代码或 Pull Request 之前，请阅读以下开发与协同规范。

Thank you for contributing to **Imomoe Sounds**! Please review these guidelines before submitting code or a Pull Request.

---

## 1. 开发环境与工具链 / Environment & Toolchain

本项目基于 Rust 2024 Edition，绑定并依赖 [Plugify](https://github.com/untrustedmodders/plugify) 与 [S2SDK](https://github.com/untrustedmodders/plugify-plugin-s2sdk)。

Built with Rust 2024 Edition against [Plugify](https://github.com/untrustedmodders/plugify) and [S2SDK](https://github.com/untrustedmodders/plugify-plugin-s2sdk):

- **Rust 工具链 / Toolchain**：请通过 `rust-toolchain.toml` 使用固定的 `1.99.1` 版本（含 `rustfmt` 与 `clippy`）：
  ```bash
  rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy
  ```
- **操作系统支持 / Supported OS**：Windows x86_64 MSVC、Linux x86_64 GNU（发布环境建议基于 Steam Runtime Sniper）。

---

## 2. 质量门禁与本地验证 / Quality Gates

所有提交必须通过以下四项标准门禁 / Every submission must pass all four checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

---

## 3. 核心设计与编码规范 / Architecture & Safety Contracts

1. **FFI 边界安全 / FFI Boundary Safety**：
   - 暴露给游戏引擎或 Plugify 的外部 C ABI 入口（`start`、`end`、`PlayCue` 及事件回调）必须使用 `catch_unwind` / `boundary` 拦截 panic，所有引擎调用必须在游戏主线程执行。
   - External C ABI entry points must catch panics with `catch_unwind` / `boundary`, and engine calls must execute on the game thread.
2. **状态与生命周期契约 / State & Lifecycle**：
   - 玩家断线清理该槽位连杀与防抖记录（个人偏好按 SteamID64 保留），换图重置播放游标与回合状态，卸载（`end`）精准注销所有命令、事件 Hook 与监听器。
   - Clear per-slot streaks/cursors cleanly on disconnect, deletion, map end, and `end`, while retaining SteamID64 `!sound` preferences.
3. **配置与 S2SDK 绑定 / Config & Bindings**：
   - 配置解析采用严苛模型（`deny_unknown_fields`）；Workshop 资源在构建地图 Manifest 时逐图验证并预缓存，不维护每客户端原生回退表。
   - Enforce strict JSONC schemas (`deny_unknown_fields`) and per-map Workshop `ReadFileVPK` validation without per-client native fallback lists.
   - 更新 `bindings/s2sdk.pplugin` 后运行 `plugify-gen -lang rust -manifest bindings/s2sdk.pplugin -output . -overwrite` 并 `cargo fmt`。

---

## 4. 提交与 PR 流程 / Pull Request Workflow

1. 基于 `main` 分支拉取特性分支并保持每次提交聚焦单一主题，切勿提交个人运行配置（`sounds.jsonc`、`player_preferences.json`）、真实社区 Workshop ID 或私有信息。
   Branch from `main`, keep commits focused, and never commit live configs, Workshop IDs, or private server details.
2. 本地四门禁全部通过后发起 Pull Request，清晰说明改动动机、影响范围及验证情况。
   Open a PR after all four local checks pass, summarizing motivation, scope, and verification.
