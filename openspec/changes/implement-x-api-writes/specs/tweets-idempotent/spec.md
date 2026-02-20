## MODIFIED Requirements
### Requirement: 投稿作成の冪等実行
`xcom-rs` は `tweets create` の実行時、スタブではなくX APIへリクエストを送信し、成功時のtweet IDを冪等結果として保存しなければならない（MUST）。

#### Scenario: 実API呼び出しの結果保存
- **Given** 利用者が `tweets create --text "hello"` を実行する
- **When** X APIが作成済みtweet IDを返す
- **Then** CLIはそのIDをレスポンスに含める
- **And** 同一 `client-request-id` 再実行時は保存済みIDを返す
