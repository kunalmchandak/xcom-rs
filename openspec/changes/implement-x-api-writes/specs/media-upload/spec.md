## MODIFIED Requirements
### Requirement: メディアアップロードコマンドの提供
`xcom-rs` は `media upload` 実行時にX APIへメディアアップロードを送信しなければならない（MUST）。

#### Scenario: 実APIアップロード成功
- **Given** 利用者が `media upload ./image.png --output json` を実行する
- **When** CLIがAPIにアップロードする
- **Then** APIが返す `media_id` が `data.media_id` に反映される
