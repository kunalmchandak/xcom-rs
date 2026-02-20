# uat-required-messages Specification (Delta)

## ADDED Requirements

### Requirement: ユーザーコンテキスト必須操作の認証エラー案内
`xcom-rs` はユーザーコンテキスト必須の API 呼び出しがアプリ専用トークンで拒否された場合、`auth_required` と `nextSteps` を返して次のアクションを提示しなければならない（MUST）。

#### Scenario: `/2/users/me` が 403 で拒否される
- **Given** `XCOM_RS_BEARER_TOKEN` が設定されている
- **And** user access token (user context) が存在しない
- **And** `GET /2/users/me` が `403` と `application-only` を含むレスポンスで失敗する
- **When** ユーザーコンテキスト必須のコマンドを実行する
- **Then** `error.code=auth_required` を返す
- **And** `error.details.nextSteps` に `xcom-rs auth login` が含まれる
- **And** `error.details.nextSteps` に `XCOM_RS_BEARER_TOKEN` への案内が含まれる
