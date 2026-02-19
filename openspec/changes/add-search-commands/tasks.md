## 1. CLI 追加とルーティング

- [x] 1.1 `search recent/users` のサブコマンドを定義する（確認: `src/cli.rs` に引数定義が追加されている）
- [x] 1.2 追加コマンドをハンドラに配線する（確認: `src/handlers/` で分岐が追加されている）

## 2. 検索処理ロジック

- [x] 2.1 recent 検索の処理を実装する（確認: `src/search/commands.rs` に対応関数が存在する）
- [x] 2.2 users 検索の処理を実装する（確認: `src/search/commands.rs` に対応関数が存在する）

## 3. API クライアントとモック

- [x] 3.1 X API 呼び出しのためのクライアントIFとモック実装を追加する（確認: `src/` 配下に trait と mock が存在する）
- [x] 3.2 search のフィクスチャを追加する（確認: テスト用データが `src/test_utils.rs` または専用モジュールに追加されている）

## 4. イントロスペクションとコスト

- [x] 4.1 `commands/schema/help` に新コマンドを追加する（確認: `src/introspection.rs` の一覧とSchemaが更新されている）
- [x] 4.2 コスト見積の operation key を追加する（確認: `src/billing/storage.rs` に `search.recent` 等が追加されている）

## 5. テスト

- [x] 5.1 recent/users 検索のユニットテストを追加する（確認: `src/search/commands.rs` の `#[cfg(test)]` にテストがある）
- [x] 5.2 CLI パーステストを追加する（確認: `src/cli.rs` のテストに新サブコマンドが含まれる）
