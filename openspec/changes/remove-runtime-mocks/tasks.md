## 1. ツイート系モックの撤廃

- [ ] 1.1 `src/tweets/client.rs` から `MockTweetApiClient` と関連フィクスチャ/テストを削除する（検証: `rg -n "MockTweetApiClient" src` が0件）
- [ ] 1.2 `TweetCommand` の既定構築からモック依存を除去し、`handle_tweets` が `HttpTweetApiClient` を注入する（検証: `src/tweets/commands/mod.rs` と `src/handlers/tweets.rs` にモック参照がない）
- [ ] 1.3 `tweets list/show/conversation` のテストを `mockito` + `XCOM_RS_API_BASE` で実APIクライアント検証に置換する（検証: `tests/` または `src/tweets/commands/*` の該当テストがHTTPモックを利用）

## 2. 擬似エラー注入の撤廃とタイムライン調整

- [ ] 2.1 `XCOM_SIMULATE_ERROR` / `XCOM_RETRY_AFTER_MS` の分岐を削除する（検証: `rg -n "XCOM_SIMULATE_ERROR|XCOM_RETRY_AFTER_MS" src` が0件）
- [ ] 2.2 `tweets` のレート制限/エラー系テストを `mockito` で再現する（検証: `tests/` にHTTPステータス検証が追加される）
- [ ] 2.3 `timeline` のHTTPベースURLをテストで差し替え可能にし、擬似エラー注入に依存しないテストへ移行する（検証: `src/timeline/commands.rs` にベースURL指定があり、`tests/` で `mockito` を使用）

## 3. テストユーティリティの分離

- [ ] 3.1 `src/test_utils.rs` を `tests/` 配下へ移動し、本番公開を削除する（検証: `src/lib.rs` から `pub mod test_utils;` が削除され、参照が `tests/` に更新される）
- [ ] 3.2 既存テストの参照先を更新する（検証: `rg -n "test_utils" tests src` で本番コード参照が0件）
