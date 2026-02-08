# separate-execution-context Tasks

- [x] 1. `src/context.rs` から `check_*` メソッドを新しい型（例: `ExecutionPolicy`）に移動する。
   - 検証: `ExecutionContext` にバリデーションメソッドが残らない。
- [x] 2. `ExecutionPolicy` を `src/policy.rs` または `src/context.rs` 内に定義する。
   - 検証: `cargo check` が通る。
- [x] 3. `src/main.rs` での呼び出しを `policy.check_*(&ctx, ...)` に変更する。
   - 検証: 既存の挙動が維持される（統合テストで確認）。
- [x] 4. 既存のテストを更新する。
   - 検証: `cargo test --verbose` が成功する。
