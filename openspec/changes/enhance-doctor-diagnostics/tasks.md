# enhance-doctor-diagnostics タスク

## 1. 診断モデルの拡張

- [x] 1.1 `DoctorDiagnostics` に `scopeCheck` と `apiProbe` を追加する（検証: `src/doctor.rs` に新フィールドが定義される）
- [x] 1.2 スコープ診断の判定ロジックを追加する（検証: 期待される不足スコープが返るテストがある）

## 2. 疎通プローブの追加（モック前提）

- [x] 2.1 プローブ用のインターフェースとモック実装を追加する（検証: テストでモックが差し替え可能）
- [x] 2.2 `--probe` 指定時のみ疎通を実行するようにワイヤリングする（検証: `src/cli.rs` と `src/handlers/doctor.rs` にフラグ利用が反映される）

## 3. テストと例の更新

- [x] 3.1 `doctor` の成功/失敗/スキップをモックでテストする（検証: `cargo test doctor` 相当が通る）
- [x] 3.2 `docs/examples.md` に `doctor --probe` の例を追加する（検証: 例が追加されている）
