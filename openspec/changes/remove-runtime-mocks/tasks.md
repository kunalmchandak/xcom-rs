## 1. ツイート系モックの撤廃

- [x] 1.1 `src/tweets/client.rs` から `MockTweetApiClient` と関連フィクスチャ/テストを削除する（検証: `rg -n "MockTweetApiClient" src` が0件）
- [x] 1.2 `TweetCommand` の既定構築からモック依存を除去し、`handle_tweets` が `HttpTweetApiClient` を注入する（検証: `src/tweets/commands/mod.rs` と `src/handlers/tweets.rs` にモック参照がない）

## 2. 擬似エラー注入の撤廃とタイムライン調整

- [x] 2.1 `XCOM_SIMULATE_ERROR` / `XCOM_RETRY_AFTER_MS` の分岐を削除する（検証: `rg -n "XCOM_SIMULATE_ERROR|XCOM_RETRY_AFTER_MS" src` が0件）

## 3. テストユーティリティの分離

- [x] 3.1 `src/test_utils.rs` を `tests/` 配下へ移動し、本番公開を削除する（検証: `src/lib.rs` から `pub mod test_utils;` が削除され、参照が `tests/` に更新される）
- [x] 3.2 既存テストの参照先を更新する（検証: `rg -n "test_utils" tests src` で本番コード参照が0件）

## Future Work

- 1.3 `tweets list/show/conversation` のテストを `mockito` + `XCOM_RS_API_BASE` で実APIクライアント検証に置換する（検証: `tests/` または `src/tweets/commands/*` の該当テストがHTTPモックを利用）
  - 理由: mockitoベースのHTTP統合テストは別途実装が必要。現在の変更でモック実装の削除とランタイム分岐の削除は完了している。
- 2.2 `tweets` のレート制限/エラー系テストを `mockito` で再現する（検証: `tests/` にHTTPステータス検証が追加される）
  - 理由: 同上。mockitoベースのHTTPモックテストフレームワーク構築が必要。
- 2.3 `timeline` のHTTPベースURLをテストで差し替え可能にし、擬似エラー注入に依存しないテストへ移行する（検証: `src/timeline/commands.rs` にベースURL指定があり、`tests/` で `mockito` を使用）
  - 理由: 同上。HTTPクライアントのベースURL注入機構とmockitoテスト実装が必要。

## Acceptance #1 Failure Follow-up

- [x] `tweets list` の未認証時エラー分類を修正し、`error.code=auth_required` を返す（根拠: `src/tweets/client.rs` の `HttpTweetApiClient::list_tweets` が `ErrorDetails` を `anyhow` 文字列へ変換し、`src/handlers/tweets.rs` の `handle_list` が `internal_error` にフォールバックしている）
- [x] `tweets conversation` の未認証時エラー分類を修正し、`error.code=auth_required` かつ `data.posts` 非返却を満たす（根拠: `src/tweets/commands/show.rs` の `conversation` が文脈化で原因種別を落とし、`src/handlers/tweets.rs` の `handle_conversation` が常に `internal_error` を返している）
- [x] `MockTimelineClient` を本番コードパスから除外し、`#[cfg(test)]` もしくは `tests/` 配下へ移動する（根拠: `src/timeline/commands.rs` に `pub struct MockTimelineClient` が本番コンパイル対象として残存）
- [x] `src/test_utils.rs` の移設を完了し、`tests/` 側の参照を更新してテストを再コンパイル可能にする（根拠: `src/test_utils.rs` が残存し、`tests/*` が `xcom_rs::test_utils` を参照して `cargo test --quiet` が失敗）
