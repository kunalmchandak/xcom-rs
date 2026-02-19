# add-shell-completions タスク

## 1. CLIと生成処理

- [x] 1.1 `completion` サブコマンドと `--shell` 引数を追加する（検証: `src/cli.rs` でパースできる）
- [x] 1.2 補完生成処理を実装しstdoutに出力する（検証: `completion --shell bash` 等が期待形式を返すテストがある）

## 2. ドキュメント更新

- [x] 2.1 READMEに補完の導入手順を追記する（検証: `README.md` に手順がある）
- [x] 2.2 docs/examplesに補完例を追加する（検証: `docs/examples.md` に例がある）
