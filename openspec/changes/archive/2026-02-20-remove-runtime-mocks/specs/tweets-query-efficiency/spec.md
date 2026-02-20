## MODIFIED Requirements

### Requirement: 取得結果の投影
`xcom-rs` は `tweets list` の結果をX APIの応答に基づいて返し、モック実装によるダミー生成を行ってはならない（MUST NOT）。

#### Scenario: 認証未設定時の失敗
- **Given** `XCOM_RS_BEARER_TOKEN` が未設定で、保存済みの認証情報も存在しない
- **When** 利用者が `tweets list --output json` を実行する
- **Then** `error.code=auth_required` が返る
- **And** `data.tweets` は返らない
