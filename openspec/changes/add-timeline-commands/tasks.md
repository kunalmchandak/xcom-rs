## 1. CLI 追加とルーティング

- [ ] 1.1 `timeline home/mentions/user` のサブコマンドを定義する（確認: `src/cli.rs` に定義が追加されている）
- [ ] 1.2 `timeline` ハンドラを追加してルーティングする（確認: `src/handlers/mod.rs` と `src/main.rs` が更新されている）

## 2. タイムライン取得ロジック

- [ ] 2.1 認証ユーザーID取得の処理を追加する（確認: `GET /2/users/me` を呼ぶ関数が存在する）
- [ ] 2.2 home/mentions/user の取得処理を実装する（確認: `src/handlers/timeline.rs` 相当が存在する）

## 3. API クライアントとモック

- [ ] 3.1 タイムライン用APIクライアントのモックを追加する（確認: モックが `src/test_utils.rs` 等に追加されている）
- [ ] 3.2 ページングトークンのフィクスチャを追加する（確認: テストで `next_token` が検証されている）

## 4. イントロスペクションとコスト

- [ ] 4.1 `commands/schema/help` に timeline コマンドを追加する（確認: `src/introspection.rs` が更新されている）
- [ ] 4.2 コスト見積の operation key を追加する（確認: `src/billing/storage.rs` に `timeline.*` が追加されている）

## 5. テスト

- [ ] 5.1 タイムライン取得のユニットテストを追加する（確認: `src/...` の `#[cfg(test)]` に追加）
- [ ] 5.2 CLI パーステストを追加する（確認: `src/cli.rs` のテストに timeline が含まれる）
