# media-upload Specification

## Purpose
メディアアップロードを追加する。
## Requirements
### Requirement: メディアアップロードコマンドの提供
`xcom-rs` は `media upload` 実行時に、解決された認証方式(Bearer または OAuth1.0a)でX APIへメディアアップロードを送信しなければならない（MUST）。

#### Scenario: OAuth1.0a でのアップロード
- **Given** OAuth1.0a の認証情報が解決されている
- **When** CLIがメディアアップロードを送信する
- **Then** `Authorization: OAuth ...` ヘッダが付与される

### Requirement: 不正パスの検出
`xcom-rs` は存在しないパスの指定を検出し、失敗を返さなければならない（MUST）。

#### Scenario: ファイルが見つからない
- **Given** 利用者が `media upload ./missing.png` を実行したとき
- **When** CLIがパスを検証するとき
- **Then** `InvalidInput` が返る

