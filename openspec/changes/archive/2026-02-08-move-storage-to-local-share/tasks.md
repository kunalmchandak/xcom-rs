# タスク

- [x] 1. auth保存先のデフォルト解決規則を`XDG_DATA_HOME`基準に変更する
   - 完了条件: `AuthStore::default_storage_path`が`XDG_DATA_HOME`と`~/.local/share`を使用する
   - 検証: `rg -n "default_storage_path\(\).*auth" -n src/auth.rs`で該当実装が更新されている

- [x] 2. budget保存先のフォールバックを`~/.local/share`に変更する
   - 完了条件: `BudgetTracker::default_storage_path`のフォールバックが`~/.local/share`になる
   - 検証: `rg -n "default_storage_path\(\).*budget" -n src/billing.rs`で該当実装が更新されている

- [x] 3. XDGパス解決のテスト期待値を更新する
   - 完了条件: `tests/xdg_paths_test.rs`でauth/budgetの期待パスが`.local/share`になっている
   - 検証: `rg -n "local/share" tests/xdg_paths_test.rs`で期待値が確認できる

- [x] 4. 仕様差分を追加する
   - 完了条件: `openspec/changes/move-storage-to-local-share/specs/storage-path-resolution/spec.md`が更新される
   - 検証: `npx @fission-ai/openspec@latest show move-storage-to-local-share --json --deltas-only`で要件が表示される

- [x] 5. 提案の検証を行う
   - 完了条件: `npx @fission-ai/openspec@latest validate move-storage-to-local-share --strict`が成功する
   - 検証: コマンドが`Validation succeeded`を出力する
