# auth-oauth2-login Specification

## ADDED Requirements
### Requirement: OAuth2.0 ログインフローの提供
`xcom-rs` は OAuth2.0 Authorization Code + PKCE で user access token を取得できる `auth login` を提供しなければならない（MUST）。

#### Scenario: ローカルコールバックでのログイン
- **Given** 利用者が `xcom-rs auth login --method local-server` を実行する
- **When** ブラウザで認可が完了し、`redirect_uri` に `code` と `state` が返る
- **Then** CLIは `code` を `token` エンドポイントへ交換し、アクセストークンを保存する
- **And** 取得した `authMode` と `scopes` が `auth status` で確認できる

### Requirement: 認証情報の保存と読み出し
`xcom-rs` は取得したアクセストークンとメタ情報を `config_dir()/xcom-rs/auth.json` に保存し、再実行時に読み出せなければならない（MUST）。

#### Scenario: 保存済みトークンの利用
- **Given** `auth.json` に有効なアクセストークンが保存されている
- **When** `xcom-rs auth status --output json` を実行する
- **Then** `authenticated=true` を返す
- **And** `authMode=oauth2` と `expiresAt` が返る

### Requirement: リフレッシュトークンによる更新
`xcom-rs` は `offline.access` により refresh token が存在する場合、期限切れのアクセストークンを自動更新できなければならない（MUST）。

#### Scenario: 期限切れ時の自動更新
- **Given** `auth.json` に期限切れのアクセストークンと refresh token が保存されている
- **When** CLIがAPI呼び出し前にトークンを解決する
- **Then** refresh token で新しいアクセストークンを取得する
- **And** 保存済みトークンを更新する

### Requirement: ログアウトと取り消し
`xcom-rs` は `auth logout` で保存済み認証情報を削除でき、`--revoke` 指定時は revoke エンドポイントを呼び出さなければならない（MUST）。

#### Scenario: 保存済み認証情報の削除
- **Given** `auth.json` に認証情報が保存されている
- **When** 利用者が `xcom-rs auth logout` を実行する
- **Then** 保存済み認証情報が削除される
- **And** `auth status` は `authenticated=false` を返す

### Requirement: 非対話時の安全失敗
`xcom-rs` は `auth login` を `--non-interactive` で実行した場合、プロンプトを出さずに構造化エラーで失敗しなければならない（MUST）。

#### Scenario: 非対話でのログイン拒否
- **Given** 利用者が `xcom-rs --non-interactive auth login` を実行する
- **When** CLIが認証フローを開始しようとする
- **Then** `error.code=auth_required` と `nextSteps` を返す
