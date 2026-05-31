# Agent Instructions

- Views live in `src/views/`.
- Reusable UI components live in `src/components/`.
- App state lives in `src/state.rs`.
- User actions live in `src/actions.rs`.
- Runtime settings schema lives in `src/settings.rs`.
- App metadata, permissions, windows, actions, and settings schema live in `Stuk.toml`.
- Prefer existing Stuk widgets before custom drawing.
- Use semantic materials (`Maris`, `Luca`, `Surface`) instead of hardcoded blur.
- Run `stuk validate` after manifest changes.
- Run `cargo test` if logic changed.
