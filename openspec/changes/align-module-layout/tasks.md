# align-module-layout Tasks

1. `src/auth.rs` を `src/auth/mod.rs`, `src/auth/models.rs`, `src/auth/storage.rs` に分割する。
   - 検証: `src/auth/` 配下にモジュールが存在し、`cargo check` が通る。
2. `src/billing.rs` を `src/billing/mod.rs`, `src/billing/models.rs`, `src/billing/storage.rs` に分割する。
   - 検証: `src/billing/` 配下にモジュールが存在し、`cargo check` が通る。
3. 既存のテストを更新する。
   - 検証: `cargo test --verbose` が成功する。
4. `AGENTS.md` や `README.md` のモジュール構成説明を更新する。
   - 検証: ドキュメントが最新の構成を反映している。
