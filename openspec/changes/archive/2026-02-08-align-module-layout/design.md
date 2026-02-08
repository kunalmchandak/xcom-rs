# align-module-layout Design

## 設計方針
- 各ドメインモジュールを `models`, `storage`, `commands` (必要に応じて) に分割する。
- `tweets` モジュールのパターンを他のモジュールにも適用する。

## 構成案
- `src/auth/` → `mod.rs`, `models.rs`, `storage.rs`
- `src/billing/` → `mod.rs`, `models.rs`, `storage.rs`

## 代替案
- 現状維持
  - 一貫性が欠如し、新規開発者の学習コストが増加する。
