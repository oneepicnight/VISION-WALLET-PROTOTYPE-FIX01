## [Unreleased]
### Fixed
- Windows sled DB lock conflicts by centralizing DB access via `cash_store::db_owned()` and avoiding concurrent opens in tests.
- Axum route captures updated to `{id}` to match axum 0.7, preventing startup panics.
- E2E tests read state via API instead of opening sled directly (Windows-safe).

### Added
- `market_cash_orders` sled tree as the canonical CASH order store.
- Migration shim `migrate_legacy_prefix()` (+ optional `cleanup_legacy_prefix()` behind `CASH_MIGRATION_DELETE_LEGACY=1`).
- Cursor pagination for `/admin/cash/orders` with base64 `<updated_at>|<id>` cursors.
- `market::cursor::{encode_cursor, decode_cursor}` with round-trip unit test.
- Startup logs show absolute sled DB path.

### Dev / DX
- Clippy config & lint gating; README lints section.
- Windows testing notes: sled patterns and route syntax.

### PR checklist
- [ ] `cargo build` (dev) passes
- [ ] `cargo clippy --all-targets --all-features` passes (or warnings acknowledged)
- [ ] `cargo test` passes (Windows + Linux)
- [ ] Verified migration logs on first boot (moved count)
- [ ] (Optional) Set `CASH_MIGRATION_DELETE_LEGACY=1` and confirmed legacy cleanup
- [ ] Frontend Orders uses cursor pagination; “Load more” works
