## 1. エラー分類の追加

- [ ] 1.1 `403` とレスポンス body のヒューリスティクスで UAT 不足を検出する分類関数を追加する（確認: `src/x_api/error.rs` もしくは共通ヘルパーに判定ロジックが追加されている）
- [ ] 1.2 `/2/users/me` 失敗時に `auth_required` が伝播するように呼び出し側を接続する（確認: `src/tweets/client.rs` / `src/timeline/commands.rs` / `src/bookmarks/commands.rs` のいずれかで `auth_required` を返す経路が実装されている）

## 2. エラーメッセージと nextSteps の整備

- [ ] 2.1 `auth_required` の `message` と `nextSteps` を UAT 不足向けに統一する（確認: 文字列が `xcom-rs auth login` と `XCOM_RS_BEARER_TOKEN` を含む）

## 3. テスト

- [ ] 3.1 `GET /2/users/me` の `403` + `application-only` を mockito で再現し、`auth_required` と `nextSteps` を検証する統合テストを追加する（確認: `tests/` 配下に新規テストがあり、`cargo test <test_name>` で通る）
