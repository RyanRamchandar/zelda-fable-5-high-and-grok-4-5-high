# Shard of the Triforce

Browser-playable 2D Zelda-like action-adventure (Rust → WASM, Canvas2D,
"Minish Cap cleanline" art direction). Act 1 polished first; Acts 2–3 follow.

Design docs live in [`docs/`](docs/):

- `docs/DECISIONS.md` — locked stack / art / audio / input / save / deploy
- `docs/ARCHITECTURE.md` — crate layout, rendering, entity model, ownership
- `docs/GAME_DESIGN.md` — Act 1 combat, overworld, dungeon, Granite Warden
- `docs/PHASE_PLAN.md` — phases 0–5 with acceptance criteria

## Run
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve            # dev, http://localhost:8080
trunk build --release  # static output in dist/
## Checks
cargo check --workspace --target wasm32-unknown-unknown
cargo clippy --workspace --target wasm32-unknown-unknown
