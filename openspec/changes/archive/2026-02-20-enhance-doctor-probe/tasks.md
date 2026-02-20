## 1. HTTPプローブの実装

- [x] 1.1 `doctor --probe` のプローブをHTTPリクエストへ置き換える（検証: `src/handlers/doctor.rs` がHTTP送信を行う）
- [x] 1.2 認証未設定時は `Skipped` と次の手順を返す（検証: `XCOM_RS_BEARER_TOKEN` 未設定時の診断テストを追加）

## 2. テスト

- [x] 2.1 モックサーバ経由のHTTPプローブテストを追加する（検証: `cargo test doctor::` が成功する）
