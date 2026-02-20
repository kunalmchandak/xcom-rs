## 1. 投稿作成・返信・スレッド

- [x] 1.1 `tweets create` をX API呼び出しに置き換え、成功レスポンスのIDをLedgerへ保存する（検証: `src/tweets/commands/create.rs` がHTTPクライアント経由になる）
- [x] 1.2 `tweets reply` を `POST /2/tweets` の返信形式に置き換える（検証: `src/tweets/commands/thread.rs` の reply がAPI送信になる）
- [x] 1.3 `tweets thread` を逐次API送信に置き換え、部分失敗時の構造化エラーを維持する（検証: thread の部分失敗テストが継続して通る）

## 2. エンゲージメントとブックマーク

- [x] 2.1 `GET /2/users/me` でユーザーIDを解決するヘルパーを追加する（検証: いいね/RT/ブックマークで共通利用される）
- [x] 2.2 `tweets like/unlike/retweet/unretweet` を実API呼び出しに置き換える（検証: `src/tweets/commands/engagement.rs` でHTTP送信が行われる）
- [x] 2.3 `bookmarks add/remove` を実API呼び出しに置き換える（検証: `src/bookmarks/commands.rs` がHTTPクライアント経由になる）

## 3. メディアアップロード

- [x] 3.1 `media upload` を `POST /2/media/upload` の実API呼び出しに置き換える（検証: `src/media/commands.rs` がHTTP送信とレスポンス解析を行う）
- [x] 3.2 失敗時のエラー分類（認証/サービス障害）を共通エラー形式へ反映する（検証: 401/503 のテストが追加されている）

## 4. テスト

- [x] 4.1 `mockito` で投稿作成/返信/スレッドのHTTPリクエスト検証テストを追加する（検証: `cargo test tweets::commands::` が成功する）
- [x] 4.2 いいね/RT/ブックマーク/メディアアップロードのHTTPモックテストを追加する（検証: それぞれのコマンド単体テストが成功する）
