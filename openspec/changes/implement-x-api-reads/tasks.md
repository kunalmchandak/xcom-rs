## 1. 検索コマンド

- [x] 1.1 `search recent` を `GET /2/tweets/search/recent` に置き換える（検証: `src/search/commands.rs` がHTTP送信になる）
- [x] 1.2 `search users` をユーザー検索APIに置き換える（検証: `src/search/commands.rs` が実APIレスポンスを解析する）

## 2. タイムライン

- [x] 2.1 `timeline home/mentions` をAPI呼び出しに置き換える（検証: `src/timeline/commands.rs` がHTTP送信になる）
- [x] 2.2 `timeline user` のhandle解決とユーザー投稿取得をAPI化する（検証: handle解決と投稿取得が分離される）

## 3. ツイート取得

- [x] 3.1 `tweets list` をAPI取得に置き換え、`--fields` を `tweet.fields` にマッピングする（検証: `src/tweets/commands/list.rs` がAPI呼び出しになる）
- [x] 3.2 `tweets show` を `GET /2/tweets/{id}` で取得する（検証: `src/tweets/commands/show.rs` がAPI送信になる）
- [x] 3.3 `tweets conversation` を `conversation_id` 検索で構成する（検証: 会話ツリーの組み立てがAPI結果ベースになる）

## 4. テスト

- [x] 4.1 検索/タイムラインのHTTPモックテストを追加する（検証: `cargo test search:: timeline::` が成功する）
- [x] 4.2 `tweets list/show/conversation` のHTTPモックテストを追加する（検証: `cargo test tweets::commands::show` が成功する）

## Acceptance #1 Failure Follow-up

- [x] `tweets list/show/conversation` の実行経路が実APIを使うように修正する（証拠: `src/tweets/commands/mod.rs` の `TweetCommand::new` が `HttpTweetApiClient::from_env()` を呼び出すように変更）
- [x] テスト用モック実装を本番コード経路から分離する（証拠: `src/search/mod.rs`, `src/timeline/mod.rs`, `src/tweets/mod.rs` で `MockClient` を `#[cfg(test)]` 配下に移動）
- [x] 追跡されているバックアップコードを削除してデッドコードを解消する（証拠: `.bak`, `.bak2`, `.bak3` ファイルを削除）
