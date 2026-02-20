## 1. 認証モデルと保存の拡張

- [ ] 1.1 OAuth1.0a認証情報モデルと後方互換のデシリアライズを追加する（検証: `src/auth/models.rs` のユニットテストで OAuth2 既存JSONと OAuth1.0a JSON の両方が読み取れること）
- [ ] 1.2 AuthStoreの認証解決優先順位と `auth status` のOAuth1.0a表示を実装する（検証: `src/auth/storage.rs` のユニットテストで環境変数優先・authMode=oauth1a・refreshable=false を確認）

## 2. OAuth1.0a ログインフロー

- [ ] 2.1 OAuth1.0aのrequest_token/authorize/access_tokenフローを追加する（検証: `mockito` を使ったユニットテストで `oauth/request_token` と `oauth/access_token` をスタブして成功を確認）
- [ ] 2.2 CLIに `auth login --mode oauth1a` を追加し、`--non-interactive` では構造化エラーを返す（検証: `src/cli.rs` と `src/handlers/auth.rs` のテストで引数パースと `auth_required` を確認）

## 3. OAuth1.0a 署名とHTTP統合

- [ ] 3.1 OAuth1.0a署名ヘッダ生成ヘルパーを追加し、HMAC-SHA1が含まれることを保証する（検証: 署名ヘッダ生成のユニットテストで `oauth_signature_method=HMAC-SHA1` を確認）
- [ ] 3.2 XApiClientでBearer/OAuth1.0aを自動切替し、OAuth1.0a時に `Authorization: OAuth ...` を送る（検証: `src/x_api/client.rs` のテストでヘッダを検証）
- [ ] 3.3 MediaアップロードがOAuth1.0a認証を利用できるようにAuth解決を統合する（検証: `src/media/commands.rs` のテストでOAuthヘッダが付与されることを確認）
