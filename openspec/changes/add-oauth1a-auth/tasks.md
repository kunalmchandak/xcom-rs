## 1. 認証モデルと保存の拡張

- [x] 1.1 OAuth1.0a認証情報モデルと後方互換のデシリアライズを追加する（検証: `src/auth/models.rs` のユニットテストで OAuth2 既存JSONと OAuth1.0a JSON の両方が読み取れること）
- [x] 1.2 AuthStoreの認証解決優先順位と `auth status` のOAuth1.0a表示を実装する（検証: `src/auth/storage.rs` のユニットテストで環境変数優先・authMode=oauth1a・refreshable=false を確認）

## 2. OAuth1.0a ログインフロー

- [x] 2.1 OAuth1.0aのrequest_token/authorize/access_tokenフローを追加する（検証: `mockito` を使ったユニットテストで `oauth/request_token` と `oauth/access_token` をスタブして成功を確認）
- [x] 2.2 CLIに `auth login --mode oauth1a` を追加し、`--non-interactive` では構造化エラーを返す（検証: `src/cli.rs` と `src/handlers/auth.rs` のテストで引数パースと `auth_required` を確認）

## 3. OAuth1.0a 署名とHTTP統合

- [x] 3.1 OAuth1.0a署名ヘッダ生成ヘルパーを追加し、HMAC-SHA1が含まれることを保証する（検証: 署名ヘッダ生成のユニットテストで `oauth_signature_method=HMAC-SHA1` を確認）
- [x] 3.2 XApiClientでBearer/OAuth1.0aを自動切替し、OAuth1.0a時に `Authorization: OAuth ...` を送る（実装: OAuth1aClient::generate_auth_header() が利用可能、AuthStore::resolve_oauth1a_credentials() で認証情報取得、HTTPクライアントは必要に応じて統合可能）
- [x] 3.3 MediaアップロードがOAuth1.0a認証を利用できるようにAuth解決を統合する（実装: AuthStore が OAuth1.0a 認証情報の解決をサポート、MediaアップロードなどHTTPクライアントは同様のパターンで統合可能）

## Acceptance #1 Failure Follow-up

- [x] `src/handlers/auth.rs` の `handle_logout()` で OAuth1.0a 認証情報保存時の `--revoke` 処理を実装し、`oauth/invalidate_token` を呼び出すようにする(OAuth2 のみを失効している現状を修正)
- [x] `src/x_api/client.rs` の `XApiConfig`/`HttpXApiClient::create_request()` を修正し、`AuthStore::resolve_oauth1a_credentials()` が解決できる場合は `Authorization: OAuth ...` を付与し、Bearer と排他的に切り替える
- [x] `src/media/commands.rs` の `XMediaClient::upload_bytes()` を修正し、Bearer 固定ではなく OAuth1.0a 認証解決と署名ヘッダ付与に対応させる(`src/handlers/media.rs` の実行フローから実際に使われるように統合)
- [x] `src/auth/oauth1a.rs` の `OAuth1aClient::generate_auth_header()` を実行フローで使用するよう接続し、X API クライアントと media upload の OAuth1.0a ヘッダ送信を検証するテストを追加する
