## ADDED Requirements

### Requirement: 取得結果の投影
`xcom-rs` は `tweets list` で `--fields` を受け付け、必要項目のみ返せなければならない（MUST）。

#### Scenario: fields指定での最小出力
- **Given** 利用者が `tweets list --fields id,text --output json` を実行する
- **When** CLIが結果を返す
- **Then** 各レコードに `id` と `text` のみが含まれる

### Requirement: 明示ページング
`xcom-rs` は `--limit` と `--cursor` による明示ページングを提供しなければならない（MUST）。

#### Scenario: cursor指定での継続取得
- **Given** 利用者が `tweets list --limit 10 --cursor CURSOR_A` を実行する
- **When** CLIが結果を返す
- **Then** 最大10件を返し、次ページ用のcursor情報を `meta.pagination` に含める

### Requirement: ndjson出力
`xcom-rs` は大量データ向けに `--output ndjson` を提供しなければならない（MUST）。

#### Scenario: ndjsonでの逐次処理
- **Given** 利用者が `tweets list --output ndjson` を実行する
- **When** CLIが結果を出力する
- **Then** 1行につき1つのJSONオブジェクトを出力する
- **And** ログはstderrにのみ出力される
