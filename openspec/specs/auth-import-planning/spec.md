# auth-import-planning Specification

## Purpose
TBD - created by archiving change add-auth-import-dry-run. Update Purpose after archive.
## Requirements
### Requirement: auth import の事前計画
auth import は実行前計画を提供することが MUST である。

#### Scenario: dry-runで保存が行われない
Given `auth import --dry-run` が実行される
When インポート対象が検証される
Then 保存ファイルは変更されず、計画の要約が返る

#### Scenario: 計画が機械可読で取得できる
Given `--output json` が指定されている
When `auth import --dry-run` が実行される
Then 実行結果には変更計画の詳細が含まれる

