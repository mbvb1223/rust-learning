# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repo is

A self-paced Rust course for a senior PHP developer. `ROADMAP.md` is the plan: Stage 1 fundamentals (lessons 01–09), Stage 2 task API (10–14), Stage 3 native/parallel (15–17), each followed by a checkpoint project planned under `projects/` (not created yet). Lesson READMEs explain Rust through PHP comparisons.

The `todo!()` bodies in `lessons/*/src` are the learner's exercises. Scaffold lessons, review, and explain; don't fill in `todo!()` unless asked.

## Toolchain: Docker only

There is no host Rust toolchain. Run everything through Compose from the repo root, with `-w` pointing at the lesson crate:

```bash
docker compose run --rm -w /workspace/lessons/NN-slug rust cargo test
docker compose run --rm -w /workspace/lessons/NN-slug rust cargo fmt --check
docker compose run --rm -w /workspace/lessons/NN-slug rust cargo clippy --all-targets -- -D warnings
docker compose run --rm -w /workspace/lessons/NN-slug rust cargo test <test_name_substring>
docker compose run --rm -w /workspace/lessons/NN-slug rust cargo test --test api   # one integration test file
docker compose run --rm -w /workspace/lessons/NN-slug rust cargo test --lib        # unit tests only
```

- `cargo test` stops at the first failing test binary; use `--no-fail-fast` or `--test <file>` to see integration tests while unit tests still fail.
- `CARGO_TARGET_DIR=/cargo-target` is a named volume (bind-mounted `target/` is slow on macOS), so there's no `target/` in lesson dirs.
- The service runs `sleep infinity`, so after `docker compose up -d` the same commands also work as `docker compose exec -w … rust cargo …`. `exec` can't publish ports, so servers still use `run --rm -p`.
- Servers (12, 14) need `-p 3000:3000`; lesson 14 also needs `-e BIND_ADDR=0.0.0.0:3000`.
- Docker builds Linux binaries; they don't run on macOS.

## Lesson layout

Each `lessons/NN-slug/` is a standalone crate (no Cargo workspace), edition 2024, with its own committed `Cargo.lock`. Add dependencies only when an exercise needs them.

- `src/lib.rs`: exercises as `todo!()` stubs; the doc comment on each stub is its spec. Unit tests live alongside. `src/main.rs` is a thin binary over the library.
- `tests/`: integration tests and fixtures; shared helpers go in `tests/common/mod.rs`.
- `README.md`: always `## Run` → `## Notes` → `## Exercise` (ordered steps) → `## Done when` (passing checks plus "You can explain" questions).
- `broken` feature (03, 07, 11, 16): `src/broken.rs` is a deliberate compile-error exercise, gated by `#[cfg(feature = "broken")]` so plain `cargo test` still builds. Check it with `--features broken`.
- Unused-parameter warnings on unimplemented stubs are expected. Never run `cargo fix` (it renames params to `_name`).

## Lesson-specific gotchas

- 13, 14: SQLx with bundled SQLite, so the first build compiles SQLite from C (minutes). `cargo run` writes git-ignored `tasks.db`. Lesson 13's `build.rs` exists so `sqlx::migrate!()` picks up new migration files.
- 14: tests use a fresh migrated in-memory DB per test (`tests/common/mod.rs`); `Config` takes an env lookup closure instead of reading `std::env` directly.
- 15: Docker runs as root, so the unreadable-directory test returns early. Run it as `nobody` with `cargo test --config 'target."cfg(unix)".runner = "setpriv --reuid=65534 --regid=65534 --clear-groups"' --lib unreadable`.
- 17: benchmark timings only mean anything with `cargo run --release`.
