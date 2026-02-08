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
