# media-upload Specification

## Purpose
メディアアップロードを追加する。
## Requirements
### Requirement: メディアアップロードコマンドの提供
`xcom-rs` は `media upload` 実行時にX APIへメディアアップロードを送信しなければならない（MUST）。

#### Scenario: 実APIアップロード成功
- **Given** 利用者が `media upload ./image.png --output json` を実行する
- **When** CLIがAPIにアップロードする
- **Then** APIが返す `media_id` が `data.media_id` に反映される

### Requirement: 不正パスの検出
`xcom-rs` は存在しないパスの指定を検出し、失敗を返さなければならない（MUST）。

#### Scenario: ファイルが見つからない
- **Given** 利用者が `media upload ./missing.png` を実行したとき
- **When** CLIがパスを検証するとき
- **Then** `InvalidInput` が返る

