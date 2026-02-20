# media-upload Specification

## MODIFIED Requirements
### Requirement: メディアアップロードコマンドの提供
`xcom-rs` は `media upload` 実行時に、解決された認証方式(Bearer または OAuth1.0a)でX APIへメディアアップロードを送信しなければならない（MUST）。

#### Scenario: OAuth1.0a でのアップロード
- **Given** OAuth1.0a の認証情報が解決されている
- **When** CLIがメディアアップロードを送信する
- **Then** `Authorization: OAuth ...` ヘッダが付与される
