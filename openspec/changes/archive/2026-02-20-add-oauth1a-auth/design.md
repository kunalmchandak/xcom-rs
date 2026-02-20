# 設計メモ: OAuth1.0a 併用の認証・送信

## 目的
OAuth2(PKCE)に加えてOAuth1.0a(3-legged)を扱えるようにし、汎用CLIとしてユーザー操作系APIをカバーする。

## 主要方針
- **認証情報は統合保存**し、`authMode`でOAuth2/OAuth1.0aを判別する。
- **HTTPリクエストの認証解決を一本化**し、BearerかOAuth1.0a署名かを自動選択する。
- **外部依存はモック優先**でテスト可能性を担保する。

## 認証情報モデル
### 保存形式
`config_dir()/xcom-rs/auth.json` を継続利用する。
- OAuth2: 既存のフィールドを保持
- OAuth1.0a: 追加フィールドを使用

例(概念):
```json
{
  "authMode": "oauth1a",
  "consumerKey": "...",
  "consumerSecret": "...",
  "accessToken": "...",
  "accessTokenSecret": "...",
  "scopes": null,
  "expiresAt": null
}
```

### 後方互換
- 既存のOAuth2のみの`auth.json`は読み込み可能にする。
- `authMode`未設定の場合はOAuth2として扱う。

## 認証解決の優先順位
1. OAuth1.0a 環境変数 (CI/非対話用)
2. OAuth2/Bearer 環境変数
3. 保存済みOAuth1.0a
4. 保存済みOAuth2

優先順位は「明示的に指定した認証方式を優先」の原則を守る。

## OAuth1.0a 署名
- `oauth1-request` 等の既存ライブラリで署名ヘッダを生成する。
- 署名対象に含めるもの:
  - HTTP method
  - 完全URL
  - クエリパラメータ
  - `application/x-www-form-urlencoded` のbody
- `multipart/form-data` のbodyは署名対象に含めない(一般的な運用に合わせる)。

## CLI 追加
- `auth login` に `--mode oauth1a|oauth2` を追加
  - デフォルトは `oauth2`
- OAuth1.0a では `request_token -> authorize -> access_token` の3段階を実装
  - `--method local-server|manual` は既存と同様に利用

## テスト方針
- OAuth1.0aエンドポイントは `mockito` でスタブ化
- 署名ヘッダは「OAuth ...」の形式と必須パラメータの有無を検証
- 認証解決の優先順位はユニットテストで検証

## リスクと緩和
- 署名エラー: ライブラリ利用 + 既知のテストベクタを用意
- 互換性: 既存OAuth2の保存形式を壊さない
