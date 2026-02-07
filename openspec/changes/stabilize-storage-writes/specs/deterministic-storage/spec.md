# deterministic-storage

## MODIFIED Requirements

### Requirement: 同一内容の保存は再書き込みしない
同一内容の保存は再書き込みしないことが MUST である。

#### Scenario: 連続保存で内容が同一
Given 直前の保存内容と同一のデータが保存対象になる
When 保存処理が実行される
Then ファイルは再書き込みされない
