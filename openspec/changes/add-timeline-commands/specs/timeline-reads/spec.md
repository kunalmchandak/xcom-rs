# timeline-reads Specification

## Purpose
ホーム/メンション/ユーザーのタイムライン取得を追加する。

## ADDED Requirements
### Requirement: ホームタイムラインの取得
`xcom-rs` は `timeline home` を提供し、認証ユーザーのホームタイムラインを取得しなければならない（MUST）。

#### Scenario: home のページング取得
- **Given** 利用者が `timeline home --limit 20` を実行したとき
- **When** CLIが結果を返すとき
- **Then** `data.tweets` が最大20件で返る
- **And** 続きがある場合 `data.meta.pagination.next_token` が返る

### Requirement: メンションタイムラインの取得
`xcom-rs` は `timeline mentions` を提供し、認証ユーザーのメンション投稿を取得しなければならない（MUST）。

#### Scenario: mentions の取得
- **Given** 利用者が `timeline mentions --limit 10` を実行したとき
- **When** CLIが結果を返すとき
- **Then** `type="timeline.mentions"` のレスポンスが返る

### Requirement: ユーザー投稿タイムラインの取得
`xcom-rs` は `timeline user <handle>` を提供し、指定ユーザーの投稿一覧を取得しなければならない（MUST）。

#### Scenario: handle 解決と投稿取得
- **Given** 利用者が `timeline user XDev --limit 5` を実行したとき
- **When** CLIが結果を返すとき
- **Then** `@XDev` のユーザーIDが解決され、`data.tweets` が返る
