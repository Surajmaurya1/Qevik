# Contributing to Spotlight for Windows

We welcome contributions! Please follow these guidelines:

## Code Quality Standards

1. **Rust:**
   - Run `cargo fmt --all -- --check`
   - Run `cargo clippy --all-targets --all-features -- -D warnings`
   - No `unwrap()` in non-test production code.
   - All public functions must have doc comments.

2. **TypeScript / React:**
   - `"strict": true` must pass (`npm run typecheck`).
   - Run `npm run lint` (`--max-warnings 0`).
   - Run `npm run format:check`.
   - Never move search, indexing, or launch logic into React.

3. **Architecture Boundaries:**
   - Strictly follow the core separation defined in `IMPLEMENTATION.md`.

## Commit Conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat(search): add fuzzy matching for short queries`
- `fix(indexer): prevent crash on inaccessible directory`
- `perf(ranking): reduce allocations in score computation`
