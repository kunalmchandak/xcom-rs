# timeline-reads Specification

## Purpose
TBD - created by archiving change add-timeline-commands. Update Purpose after archive.
## Requirements
### Requirement: ホームタイムラインの取得
`xcom-rs` は `timeline home` 実行時にX APIからホームタイムラインを取得しなければならない（MUST）。

#### Scenario: API由来のhome取得
- **Given** 利用者が `timeline home --limit 20` を実行する
- **When** CLIがAPIから結果を取得する
- **Then** `data.tweets` はAPIの結果に基づく

### Requirement: メンションタイムラインの取得
`xcom-rs` は `timeline mentions` 実行時にX APIからメンションを取得しなければならない（MUST）。

#### Scenario: API由来のmentions取得
- **Given** 利用者が `timeline mentions --limit 10` を実行する
- **When** CLIがAPIから結果を取得する
- **Then** `type="timeline.mentions"` のレスポンスが返る

### Requirement: ユーザー投稿タイムラインの取得
`xcom-rs` は `timeline user <handle>` 実行時にX APIから投稿を取得しなければならない（MUST）。

#### Scenario: handle解決とAPI取得
- **Given** 利用者が `timeline user XDev --limit 5` を実行する
- **When** CLIがAPIから結果を取得する
- **Then** `data.tweets` が返る

