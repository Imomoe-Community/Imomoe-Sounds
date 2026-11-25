# Imomoe-Server-Sounds

Standalone CS2 Plugify combat/tactical sound plugin (`imomoe_sounds`) with JSONC + JSON Schema config and optional `imomoe_chat_prefix` integration. Read [ARCHITECTURE](docs/ARCHITECTURE.md) and [ACCEPTANCE](docs/ACCEPTANCE.md) before changing playback, precache, or exported ABI behavior.

## Workspace & hygiene

- Standalone clones default to `<repo>/.work/`. When `IMOMOE_WORK_ROOT` is set, place shared caches, task worktrees (`$IMOMOE_WORK_ROOT/worktrees/Imomoe-Server-Sounds-<task>`), and disposable probes under it instead of creating repo-local `.work/` or committing ad-hoc scripts (`smoke*.py`, `drive-*.ps1`, `*_minimal.*`).
- Keep personal/community sound packs and server presets outside this repository; shipped JSONC defaults and examples must remain community-agnostic.

## Engineering & verification

- Inspect sibling call paths before patching; fix root causes rather than adding symptom workarounds. Apply one-in, one-out closeout to superseded tests, fixtures, and docs in the same diff.
- Preserve natural numeric clip ordering (`2` before `10`) and silent fallback when optional companions (`imomoe_chat_prefix`) are absent.
- Verify changes with `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and runtime `s2sdk` descriptor checks when bindings change.
