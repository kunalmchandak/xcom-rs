## 1. 検索コマンド

- [ ] 1.1 `search recent` を `GET /2/tweets/search/recent` に置き換える（検証: `src/search/commands.rs` がHTTP送信になる）
- [ ] 1.2 `search users` をユーザー検索APIに置き換える（検証: `src/search/commands.rs` が実APIレスポンスを解析する）

## 2. タイムライン

- [ ] 2.1 `timeline home/mentions` をAPI呼び出しに置き換える（検証: `src/timeline/commands.rs` がHTTP送信になる）
- [ ] 2.2 `timeline user` のhandle解決とユーザー投稿取得をAPI化する（検証: handle解決と投稿取得が分離される）

## 3. ツイート取得

- [ ] 3.1 `tweets list` をAPI取得に置き換え、`--fields` を `tweet.fields` にマッピングする（検証: `src/tweets/commands/list.rs` がAPI呼び出しになる）
- [ ] 3.2 `tweets show` を `GET /2/tweets/{id}` で取得する（検証: `src/tweets/commands/show.rs` がAPI送信になる）
- [ ] 3.3 `tweets conversation` を `conversation_id` 検索で構成する（検証: 会話ツリーの組み立てがAPI結果ベースになる）

## 4. テスト

- [ ] 4.1 検索/タイムラインのHTTPモックテストを追加する（検証: `cargo test search:: timeline::` が成功する）
- [ ] 4.2 `tweets list/show/conversation` のHTTPモックテストを追加する（検証: `cargo test tweets::commands::show` が成功する）
