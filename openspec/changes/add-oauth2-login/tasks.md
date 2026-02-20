## 1. 認証情報の保存基盤

- [ ] 1.1 OAuth2認証情報の保存モデルを追加する（`access_token`/`refresh_token`/`expires_at`/`scopes`/`auth_mode`）
  - 検証: `src/auth/*` にモデル定義が追加され、JSONシリアライズ/デシリアライズの単体テストがある
- [ ] 1.2 `config_dir()/xcom-rs/auth.json` への読み書きを実装する
  - 検証: 読み書きユニットテストで `auth.json` が生成・復元される

## 2. OAuth2.0 ログインフロー

- [ ] 2.1 PKCEヘルパー（code_verifier/code_challenge）を実装し、S256生成をテストする
  - 検証: ユニットテストで `code_challenge` が SHA256 + base64url(no padding) になる
- [ ] 2.2 authorize URL 生成ロジックを実装する（state/redirect_uri/scope含む）
  - 検証: ユニットテストでURLパラメータが期待通りに構成される
- [ ] 2.3 `auth login` の `local-server` 方式を実装する（`/callback` 受信で code/state を取得）
  - 検証: ローカルHTTP要求を使ったテストで `code` と `state` が正しく解釈される
- [ ] 2.4 `auth login` の `manual` 方式を実装する（リダイレクトURL貼り付け解析）
  - 検証: URL解析のユニットテストで `code` と `state` を取得できる
- [ ] 2.5 `token` エンドポイントへの交換処理を実装する（public/confidential 両対応）
  - 検証: mockito で `POST /2/oauth2/token` のフォームが正しいことを確認するテスト

## 3. Refresh / Revoke / Logout

- [ ] 3.1 refresh token による更新処理を実装する
  - 検証: mockito で `grant_type=refresh_token` のリクエストを検証するテスト
- [ ] 3.2 `auth logout` を実装し、保存済み認証情報を削除する
  - 検証: `auth.json` が削除されるテスト
- [ ] 3.3 `--revoke` 指定時の revoke リクエストを実装する
  - 検証: mockito で `POST /2/oauth2/revoke` の呼び出しが確認できるテスト

## 4. トークン解決と統合

- [ ] 4.1 トークン解決ロジックを実装する（環境変数優先、保存済みトークン、期限切れ時のrefresh）
  - 検証: ユニットテストで解決順序とrefresh動作を確認する
- [ ] 4.2 X APIクライアントが新しい解決ロジックを使うように更新する
  - 検証: `src/x_api/*` で AuthStore 経由のトークン利用が確認できる
- [ ] 4.3 `auth status` を拡張する（authMode/scopes/expiresAt/refreshable）
  - 検証: `auth status --output json` のスナップショットテストで新フィールドを確認

## 5. CLI配線と非対話ガード

- [ ] 5.1 `auth login` / `auth logout` のCLIサブコマンドを追加する
  - 検証: `src/cli.rs` と `handlers/auth.rs` でルーティングされている
- [ ] 5.2 `--non-interactive` で `auth login` が `auth_required` を返す
  - 検証: 非対話モードのテストで `error.code=auth_required` が返る
