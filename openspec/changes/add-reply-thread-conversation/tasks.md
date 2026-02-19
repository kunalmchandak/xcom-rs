## 1. CLI 追加とルーティング

- [ ] 1.1 `tweets reply/thread/show/conversation` のサブコマンドを定義する（確認: `src/cli.rs` に引数定義が追加されている）
- [ ] 1.2 追加コマンドをハンドラに配線する（確認: `src/handlers/tweets.rs` で分岐が追加されている）

## 2. 投稿・会話取得ロジック

- [ ] 2.1 返信/スレッド/投稿取得のコマンド処理を実装する（確認: `src/tweets/commands.rs` に対応関数が存在する）
- [ ] 2.2 会話取得で `conversation_id` を検索しツリーを再構成する（確認: 会話再構成関数が `src/tweets/commands.rs` で呼ばれる）

## 3. API クライアントとモック

- [ ] 3.1 X API 呼び出しのためのクライアントIFとモック実装を追加する（確認: `src/tweets/` 配下に trait と mock が存在する）
- [ ] 3.2 モック用フィクスチャで `conversation_id` の再構成を再現する（確認: テスト用データが `src/test_utils.rs` または専用モジュールに追加されている）

## 4. イントロスペクションとコスト

- [ ] 4.1 `commands/schema/help` に新コマンドを追加する（確認: `src/introspection.rs` の一覧とSchemaが更新されている）
- [ ] 4.2 コスト見積の operation key を追加する（確認: `src/billing/storage.rs` に `tweets.reply` 等が追加されている）

## 5. テスト

- [ ] 5.1 返信/スレッド/会話取得のユニットテストを追加する（確認: `src/tweets/commands.rs` の `#[cfg(test)]` にテストがある）
- [ ] 5.2 CLI パーステストを追加する（確認: `src/cli.rs` のテストに新サブコマンドが含まれる）
