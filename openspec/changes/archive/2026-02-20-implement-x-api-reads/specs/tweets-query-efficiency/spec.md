## MODIFIED Requirements
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
