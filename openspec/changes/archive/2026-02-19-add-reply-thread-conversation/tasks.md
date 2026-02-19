## 1. CLI 追加とルーティング

- [x] 1.1 `tweets reply/thread/show/conversation` のサブコマンドを定義する（確認: `src/cli.rs` に引数定義が追加されている）
- [x] 1.2 追加コマンドをハンドラに配線する（確認: `src/handlers/tweets.rs` で分岐が追加されている）

## 2. 投稿・会話取得ロジック

- [x] 2.1 返信/スレッド/投稿取得のコマンド処理を実装する（確認: `src/tweets/commands.rs` に対応関数が存在する）
- [x] 2.2 会話取得で `conversation_id` を検索しツリーを再構成する（確認: 会話再構成関数が `src/tweets/client.rs` で実装され `src/tweets/commands.rs` で呼ばれる）

## 3. API クライアントとモック

- [x] 3.1 X API 呼び出しのためのクライアントIFとモック実装を追加する（確認: `src/tweets/client.rs` に `TweetApiClient` trait と `MockTweetApiClient` が存在する）
- [x] 3.2 モック用フィクスチャで `conversation_id` の再構成を再現する（確認: `MockTweetApiClient::with_conversation_fixture()` が `src/tweets/client.rs` に追加されている）

## 4. イントロスペクションとコスト

- [x] 4.1 `commands/schema/help` に新コマンドを追加する（確認: `src/introspection.rs` の一覧とSchemaが更新されている）
- [x] 4.2 コスト見積の operation key を追加する（確認: `src/billing/storage.rs` に `tweets.reply`/`tweets.thread`/`tweets.show`/`tweets.conversation` が追加されている）

## 5. テスト

- [x] 5.1 返信/スレッド/会話取得のユニットテストを追加する（確認: `src/tweets/commands.rs` の `#[cfg(test)]` にテストがある）
- [x] 5.2 CLI パーステストを追加する（確認: `src/cli.rs` のテストに新サブコマンドが含まれる）

## Acceptance #1 Failure Follow-up

- [x] `tweets conversation` のレスポンスに `data.conversation_id` を含める（`src/tweets/models.rs` の `ConversationResult`、`src/tweets/client.rs` の `fetch_conversation`、`src/introspection.rs` の `tweets conversation` output schema を更新し、`src/tweets/commands.rs` のテストで検証する）
- [x] `tweets thread` 失敗時に `failedIndex` と `createdTweetIds` を含む構造化エラーを返す（`src/tweets/commands.rs::TweetCommand::thread` で部分失敗情報を保持する専用エラーを実装し、`src/handlers/tweets.rs::handle_thread` で error details に反映、失敗系テストを追加する）
