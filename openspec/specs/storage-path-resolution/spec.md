# storage-path-resolution Specification

## Purpose
TBD - created by archiving change support-xdg-paths. Update Purpose after archive.
## Requirements
### Requirement: 保存先パスの解決規則
保存先パスの解決はXDG環境変数を優先することが MUST である。

#### Scenario: XDG環境変数が設定されている場合の解決
Given `XDG_CONFIG_HOME` または `XDG_DATA_HOME` が設定されている
When 認証/予算の保存先パスを解決する
Then XDGの値が優先され、従来の固定パスは使われない

