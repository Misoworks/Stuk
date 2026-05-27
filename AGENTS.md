# Agent Instructions

- Framework crates live in `crates/`.
- Public app-facing API belongs in `crates/stuk`.
- Runtime, view tree, component model, and lifecycle code belong in `crates/stuk-core`.
- Layout primitives belong in `crates/stuk-layout`.
- Display-list and renderer code belong in `crates/stuk-render`.
- Built-in widget builders belong in `crates/stuk-widgets`.
- Platform windowing and OS integration belong in `crates/stuk-platform`.
- App metadata, permissions, windows, actions, and settings schema parsing belong in `crates/stuk-manifest`.
- CLI behavior belongs in `crates/stuk-cli`.
- Prefer semantic materials (`Maris`, `Luca`, `Surface`) over hardcoded blur behavior.
- Run `cargo fmt`, `cargo build --workspace`, and `cargo test --workspace` after code changes.
