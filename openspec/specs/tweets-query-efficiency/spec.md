# tweets-query-efficiency Specification

## Purpose
TBD - created by archiving change plan-idempotent-tweet-operations. Update Purpose after archive.
## Requirements
### Requirement: 取得結果の投影
`xcom-rs` は `tweets list` の結果をX APIの応答に基づいて返し、モック実装によるダミー生成を行ってはならない（MUST NOT）。

#### Scenario: 認証未設定時の失敗
- **Given** `XCOM_RS_BEARER_TOKEN` が未設定で、保存済みの認証情報も存在しない
- **When** 利用者が `tweets list --output json` を実行する
- **Then** `error.code=auth_required` が返る
- **And** `data.tweets` は返らない

### Requirement: 明示ページング
`xcom-rs` は `tweets list` のページング情報をAPIのトークンから反映しなければならない（MUST）。

#### Scenario: APIトークン反映
- **Given** 利用者が `tweets list --limit 10 --cursor CURSOR_A` を実行する
- **When** CLIがAPIから結果を取得する
- **Then** 次ページ情報が `meta.pagination` に反映される

### Requirement: ndjson出力
`xcom-rs` は大量データ向けに `--output ndjson` を提供しなければならない（MUST）。

#### Scenario: ndjsonでの逐次処理
- **Given** 利用者が `tweets list --output ndjson` を実行する
- **When** CLIが結果を出力する
- **Then** 1行につき1つのJSONオブジェクトを出力する
- **And** ログはstderrにのみ出力される

