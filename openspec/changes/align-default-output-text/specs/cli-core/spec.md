# cli-core Specification Delta

## MODIFIED Requirements
### Requirement: 失敗レスポンスの出力形式整合
`xcom-rs` は `--output` 未指定時、引数解釈失敗や `--log-format` 不正などの早期エラーも `text` を既定出力形式として返さなければならない（MUST）。また、`--output` が明示された場合は早期エラーでも指定形式を尊重しなければならない（MUST）。

#### Scenario: 既定出力での早期エラー
- **Given** 利用者が `xcom-rs --log-format invalid commands` を `--output` 未指定で実行したとき
- **When** CLI がエラーを返すとき
- **Then** エラーは `text` 形式で出力される
- **And** CLI は終了コード `2` を返す

#### Scenario: サブコマンド未指定の早期エラー
- **Given** 利用者が `xcom-rs auth` を `--output` 未指定で実行したとき
- **When** CLI がエラーを返すとき
- **Then** エラーは `text` 形式で出力される
- **And** CLI は終了コード `2` を返す

#### Scenario: 明示出力での早期エラー
- **Given** 利用者が `xcom-rs --output json --log-format invalid commands` を実行したとき
- **When** CLI がエラーを返すとき
- **Then** エラーは JSON の `Envelope` 形式で出力される
- **And** CLI は終了コード `2` を返す

#### Scenario: 不正な出力形式の早期エラー
- **Given** 利用者が `xcom-rs auth --output txt` を実行したとき
- **When** CLI がエラーを返すとき
- **Then** エラーは JSON の `Envelope` 形式で出力される
- **And** CLI は終了コード `2` を返す
