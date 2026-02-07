# 設計ドキュメント: XDG準拠の保存パス解決

## 概要

認証ストレージと予算トラッカーの保存先をXDG Base Directory仕様に準拠させる実装。

## 設計決定

### パス解決ロジック

#### 認証ストレージ (auth.json)
- XDG準拠: `$XDG_CONFIG_HOME/xcom-rs/auth.json`
- フォールバック: `$HOME/.config/xcom-rs/auth.json`

ロジック:
```rust
if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
    PathBuf::from(xdg_config).join("xcom-rs")
} else {
    PathBuf::from(home).join(".config").join("xcom-rs")
}
```

#### 予算トラッカー (budget.json)
- XDG準拠: `$XDG_DATA_HOME/xcom-rs/budget.json`
- フォールバック: `$HOME/.config/xcom-rs/budget.json`

ロジック:
```rust
if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
    PathBuf::from(xdg_data).join("xcom-rs")
} else {
    PathBuf::from(home).join(".config").join("xcom-rs")
}
```

### 変更箇所

1. **src/auth.rs**
   - `AuthStore::default_storage_path()` を更新
   - XDG_CONFIG_HOME を優先的に参照

2. **src/billing.rs**
   - `BudgetTracker::default_storage_path()` を更新
   - XDG_DATA_HOME を優先的に参照

### テスト戦略

#### ユニットテスト
- XDG変数が設定されている場合のパス解決
- XDG変数が設定されていない場合のフォールバック
- 環境変数の競合を防ぐためMutexで保護

#### 結合テスト (tests/xdg_paths_test.rs)
1. `test_auth_storage_respects_xdg_config_home`
   - XDG_CONFIG_HOME設定時に正しいパスが使用される
   - 認証データのインポート/読み取りが動作する

2. `test_billing_storage_respects_xdg_data_home`
   - XDG_DATA_HOME設定時に正しいパスが使用される
   - 予算追跡ファイルが正しい場所に作成される

3. `test_fallback_to_default_path_without_xdg`
   - XDG変数未設定時にフォールバックパスが使用される

## 後方互換性

- 既存のデータ移行は非スコープ（手動移行が必要）
- XDG変数が設定されていない環境では従来通り動作

## 検証項目

✅ XDG_CONFIG_HOME設定時、auth.jsonが該当パスに保存される
✅ XDG_DATA_HOME設定時、budget.jsonが該当パスに保存される
✅ XDG変数未設定時、~/.config/xcom-rs/にフォールバックする
✅ すべてのユニットテストと結合テストがパスする
✅ cargo fmt / cargo clippy がクリーンである
✅ cargo build --release が成功する
