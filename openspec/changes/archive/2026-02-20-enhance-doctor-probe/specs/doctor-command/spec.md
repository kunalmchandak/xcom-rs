## MODIFIED Requirements
### Requirement: 診断コマンドの提供
`xcom-rs` は `doctor --probe` 指定時にHTTPプローブを実行し、結果を返さなければならない（MUST）。

#### Scenario: HTTPプローブの実行
- **Given** 利用者が `xcom-rs doctor --probe` を実行する
- **When** CLIがAPIへHTTPリクエストを送る
- **Then** `apiProbe.httpStatus` と `apiProbe.durationMs` が返る
