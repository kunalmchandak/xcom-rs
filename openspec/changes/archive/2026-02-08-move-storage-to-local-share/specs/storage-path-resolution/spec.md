# storage-path-resolution

## MODIFIED Requirements

### Requirement: 保存先パスの解決規則

認証情報と予算トラッカーの保存先はユーザーデータ領域に解決されなければならない (MUST)。

#### Scenario: XDG_DATA_HOMEが設定されている場合の解決
- Given `XDG_DATA_HOME`が設定されている
- When `auth.json`または`budget.json`の保存先を解決する
- Then `XDG_DATA_HOME/xcom-rs/<file>.json`が選択される

#### Scenario: XDG環境変数が未設定の場合のフォールバック
- Given `XDG_DATA_HOME`が設定されていない
- When `auth.json`の保存先を解決する
- Then `~/.local/share/xcom-rs/auth.json`が選択される
- And `budget.json`の保存先を解決する
- Then `~/.local/share/xcom-rs/budget.json`が選択される

## Implementation

### Authentication Storage

`AuthStore::default_storage_path()` in `src/auth.rs` implements the following resolution:

1. If `XDG_DATA_HOME` is set: `$XDG_DATA_HOME/xcom-rs/auth.json`
2. Otherwise: `~/.local/share/xcom-rs/auth.json`

### Budget Storage

`BudgetTracker::default_storage_path()` in `src/billing.rs` implements the following resolution:

1. If `XDG_DATA_HOME` is set: `$XDG_DATA_HOME/xcom-rs/budget.json`
2. Otherwise: `~/.local/share/xcom-rs/budget.json`

### Cross-Platform Support

On Windows, if `HOME` is not available, the implementation falls back to `USERPROFILE`.

### Tests

- `src/auth.rs`: Unit tests verify XDG_DATA_HOME respect and fallback behavior
- `src/billing.rs`: Unit tests verify XDG_DATA_HOME respect and fallback behavior
- `tests/xdg_paths_test.rs`: Integration tests verify end-to-end path resolution
