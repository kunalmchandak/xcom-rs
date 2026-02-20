# tweets-query-efficiency Specification

## Purpose
TBD - created by archiving change plan-idempotent-tweet-operations. Update Purpose after archive.
## Requirements
### Requirement: 取得結果の投影
`xcom-rs` は `tweets list --fields` を実行したとき、APIの `tweet.fields` へマッピングして取得しなければならない（MUST）。

#### Scenario: APIフィールド指定
- **Given** 利用者が `tweets list --fields id,text --output json` を実行する
- **When** CLIがAPIへリクエストを送る
- **Then** `tweet.fields=id,text` がAPI要求に含まれる

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

