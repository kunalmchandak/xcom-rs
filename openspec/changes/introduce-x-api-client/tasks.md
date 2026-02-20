## 1. クライアント基盤の追加

- [x] 1.1 `src/x_api` モジュールを追加し、`XApiClient` トレイトと `HttpXApiClient` 実装を定義する（検証: `src/x_api/mod.rs` と関連ファイルが追加されている）
- [x] 1.2 ベースURLと共通ヘッダー（`Authorization`/`User-Agent`）を構成できる設定構造体を用意する（検証: `XCOM_RS_API_BASE` を参照するコードがある）

## 2. エラー分類とレスポンス処理

- [x] 2.1 HTTPステータス/ヘッダーから `ClassifiedError` へマッピングするユーティリティを実装する（検証: 429/401/403/5xx を個別に分類する関数が追加されている）
- [x] 2.2 JSONレスポンスのデシリアライズと失敗時メッセージ整形を共通化する（検証: `Result<T>` へ変換する共通関数が追加されている）

## 3. テストとモック

- [x] 3.1 `mockito` を用いたHTTPリクエスト検証テストを追加する（検証: `cargo test x_api::` が成功する）
- [x] 3.2 `Retry-After` / `x-rate-limit-reset` の解析テストを追加する（検証: 429時の `retryAfterMs` を確認するテストがある）

## Acceptance #1 Failure Follow-up

- [x] `HttpXApiClient` が実行フローに統合されておらずデッドコードになっているため、CLIの実経路で `XApiConfig`/`HttpXApiClient` を生成して利用する（証拠: `src/main.rs` に `x_api` 参照がなく、`HttpXApiClient` の利用が `src/x_api/client.rs` 内に限定されている）。
- [x] 実行時デフォルトがモック実装のままのため、少なくとも本変更対象のX API呼び出し経路で `MockTweetApiClient` ではなく実HTTPクライアントを配線する（証拠: `TweetCommand::new` が `MockTweetApiClient::new()` を使用している `src/tweets/commands/mod.rs:38`）。
