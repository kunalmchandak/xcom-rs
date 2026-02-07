1. 保存先解決のヘルパーを追加する
   - 変更箇所: `src/auth.rs`, `src/billing.rs`
   - 検証: XDG変数の有無でパスが変わるユニットテストを追加
2. 認証ストレージのパス解決をXDG対応に変更する
   - 変更箇所: `src/auth.rs`
   - 検証: `AuthStore::default_storage_path()` がXDG優先になる
3. 予算トラッカーのパス解決をXDG対応に変更する
   - 変更箇所: `src/billing.rs`
   - 検証: `BudgetTracker::default_storage_path()` がXDG優先になる
4. 結合テストを追加する
   - 変更箇所: `tests/`
   - 検証: `XDG_*` を設定した状態で `auth status` / `billing estimate` がそのパスを使用する
