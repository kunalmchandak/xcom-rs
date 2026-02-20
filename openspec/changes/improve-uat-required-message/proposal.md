# 提案: UAT未設定時のエラーメッセージ改善

## 背景
`XCOM_RS_BEARER_TOKEN` が設定されていても、OAuth2 user access token (user context) が無い場合は `/2/users/me` を含むユーザーコンテキスト必須の操作が 403 で失敗します。現状のエラーは `Authorization failed` や `InternalError` に寄っており、原因と対処が分かりにくい状態です。

## 目的
- Bearer トークンがアプリケーション専用で、ユーザー認証が必要なコマンドを実行できない状況を明確に伝える。
- 次のアクション（`auth login` または UAT の設定）が分かるメッセージと `nextSteps` を返す。
- 非対話モードでも構造化エラーが一貫して返る。

## スコープ
- `/2/users/me` を前提とするユーザーコンテキスト必須コマンドのエラーメッセージ改善。
- 403 応答の body を手掛かりに、UAT 不足を判定して `auth_required` を返す。
- エラー応答に `nextSteps` を追加する。

## 非スコープ
- OAuth2 ログインフロー自体の実装（`auth login` の追加など）。
- 実際の API 連携の挙動変更や権限付与の仕様変更。

## 期待される振る舞い
- UAT が必要なコマンドでアプリ専用トークンが使われた場合、`error.code=auth_required` と `nextSteps` を返す。
- `nextSteps` には `xcom-rs auth login` や UAT 設定方法が含まれる。

## 参考
- 既存の `auth_required` エラーと `nextSteps` の出力形式に合わせる。
- 403 応答の body に `application-only` / `user context` が含まれる場合を判定材料とする。
