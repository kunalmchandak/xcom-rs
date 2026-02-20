## Context

現状の `xcom-rs` は `XCOM_RS_BEARER_TOKEN` を環境変数から読むだけで、OAuth2.0 user access token の取得・保存・更新ができません。そのためユーザコンテキストが必須のAPI（例: `/2/users/me`、`/2/users/:id/tweets`）で403が発生します。加えて `auth status` は環境変数の有無だけを返すため、実際にユーザ認証が成立しているかを判別しづらい状態です。

## Goals / Non-Goals

**Goals:**
- OAuth2.0 Authorization Code + PKCE による user access token の取得・保存・更新をCLIに実装する
- 保存済みトークンをX APIクライアントが利用できるようにし、ユーザコンテキスト必須の操作が可能になる
- `auth status` がトークンの有効性/期限/更新可否を返せるようにする

**Non-Goals:**
- OAuth1.0a のサポート
- Device Code Flow など他のOAuthフローの追加
- 実運用での認可画面のUX最適化（最小限のローカルコールバック/手動入力に留める）

## Decisions

1. **PKCE は S256 を既定にする**
   - 理由: Xの公式ドキュメント推奨。`plain` はテスト/互換用途のみ。
   - 代替案: `plain` 既定 → セキュリティ上の理由で不採用。

2. **`auth login` は `local-server` と `manual` の2方式**
   - 理由: CLIの非対話/ヘッドレス環境でも手動入力で対応できるようにする。
   - 代替案: `local-server` のみ → SSH/コンテナ環境で詰みやすい。

3. **トークン保存は `config_dir()/xcom-rs/auth.json` にJSON形式で行う**
   - 理由: 既存の `AuthStore` を拡張しやすく、依存追加が不要。
   - 代替案: OSキーチェーン → 実装コストが高く、Rust依存が増えるため今回は見送り。

4. **トークン解決の優先度は「環境変数 > 保存済み」**
   - 理由: CI/自動実行で明示的に注入したトークンを確実に優先したい。
   - 代替案: 保存済みを優先 → 既存利用者の動作が変わるため不採用。

5. **更新可能な場合は自動で refresh を試みる**
   - 理由: `offline.access` 取得時に UX を改善し、期限切れによる失敗を減らす。
   - 代替案: 失効時に明示エラーのみ → 既に refresh token がある場合に利便性が低い。

## Risks / Trade-offs

- **[Risk] 認証トークンの保存がローカルに残る** → 可能な限りファイル権限を絞り、ログに秘匿情報を出さない。
- **[Risk] X側の仕様差異（client_id必須/不要）** → confidential client の場合は Basic auth を優先し、必要に応じてclient_id送信も許容。
- **[Trade-off] 実API連携テストが外部依存になる** → mockitoを用いたスタブテストで検証し、実API検証はFuture Workに回す。

## Migration Plan

1. `auth login` を追加し、保存済み認証情報を利用する経路を実装
2. `x-api-client` のトークン解決を更新
3. `auth status` を拡張
4. 既存の `XCOM_RS_BEARER_TOKEN` 利用は引き続き有効（後方互換）

## Open Questions

- `auth login` 実装時に `auth status` が返すべき `authMode` の名称（例: `oauth2` / `env_bearer`）
