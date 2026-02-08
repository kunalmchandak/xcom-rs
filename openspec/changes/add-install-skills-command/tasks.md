# add-install-skills-command Tasks

- [x] 1. `install-skills` サブコマンドの CLI 定義を追加する
  - 検証: `xcom-rs commands --output json` に `install-skills` が含まれることを確認

- [x] 2. `install-skills` の入力スキーマとヘルプ出力を整備する
  - 検証: `xcom-rs schema --command install-skills --output json-schema` が取得できることを確認

- [x] 3. 埋め込みスキル探索ロジックを実装する（`skills/` などのローカルディレクトリを対象）
  - 検証: 単体テストで探索対象ディレクトリからスキルが列挙されることを確認

- [x] 4. `skills/` 配下に `skills/<skill-name>/SKILL.md` 形式でスキル定義を作成する
  - 検証: 追加した各スキルで `SKILL.md` が存在し、読み込み対象として認識されることを確認

- [x] 5. インストール先の解決とファイル配置を実装する（プロジェクト/グローバル、エージェント別リンク/コピー）
  - 検証: `cargo test` のユニットテストで `.agents/skills` とエージェント別パスにファイルが作成されることを確認

- [x] 6. `--skill` 指定時のスキル解決を実装し、`skills/<skill-name>/SKILL.md` を優先的に参照する
  - 検証: `xcom-rs install-skills --skill <skill-name> --non-interactive --json` で指定スキルのみが解決されることを確認

- [x] 7. 非対話モードと JSON 出力の結果整形を実装する
  - 検証: `xcom-rs install-skills --json --non-interactive` の出力で共通 Envelope の `data.installed_skills[]` が含まれることを確認

## Acceptance #1 Failure Follow-up

- [x] `src/handlers/skills.rs` の `handle_install_skills` で `--non-interactive` 単独実行を `INTERACTION_REQUIRED` で失敗させているため、`--non-interactive` または `--yes` のどちらかでプロンプト省略して実行継続するよう修正する（現状: `cargo run -- install-skills --output json --non-interactive` が失敗）。
  - 検証: `cargo run -- install-skills --output json --non-interactive` が成功し、スキルがインストールされることを確認済み
- [x] `data.installed_skills[]` の出力キーが仕様不一致（現状 `skill` / `agent_paths`）なので、仕様の `name` / `target_paths` を返すよう `src/skills/models.rs`・`src/handlers/skills.rs`・`src/introspection.rs` のスキーマ/シリアライズを揃えて修正する。
  - 検証: JSON 出力で `name` と `target_paths` が正しく出力されることを確認済み
  
## 完了確認

すべてのタスクが完了しました:
- `install-skills` サブコマンドの実装と統合
- 埋め込みスキルの探索とインストール機能
- `--non-interactive` モードでの自動確認
- JSON 出力での正しいフィールド名(`name`, `target_paths`)
- すべての単体テストおよび統合テストがパス
- コードフォーマットと lint チェックが成功
