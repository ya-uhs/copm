# CLAUDE.md

## ビルド・テスト

```bash
cargo build          # デバッグビルド
cargo test           # 全テスト実行
cargo build --release
```

テストはすべて `tests/integration_test.rs` に書く（ユニットテストは不要）。

## アーキテクチャ

```
src/
├── cli/args.rs              # clap derive によるCLI定義
├── commands/                # 各サブコマンドの実装
│   ├── init.rs              # copm init（tools選択 → copm.json生成）
│   ├── install.rs           # copm install（fetch → detect → install → lock更新）
│   ├── uninstall.rs         # copm uninstall（installed_filesベースで削除）
│   └── list.rs              # copm list（全ターゲット型のインストール先を走査）
├── config/
│   ├── copm_json.rs         # copm.json（tools, dependencies）
│   └── lock.rs              # copm.lock（targets, installed_files）
├── manifest/
│   └── package_manifest.rs  # detect_from_dir()：ファイル構造からターゲット型を自動判定
├── fetcher/
│   └── git.rs               # parse_package_spec() / fetch_package()（tarball or git clone）
├── installer/
│   ├── mod.rs               # install_targets() / uninstall_targets()：型別ディスパッチ
│   ├── copilot.rs           # 全インストーラ本体（skill / agents / prompts / instructions）
│   └── claude_plugin.rs     # 旧 claude-plugin 型（後方互換のみ）
├── paths.rs                 # 全インストール先パスの定義
└── error.rs                 # CopmError（thiserror）
```

## データフロー（install）

```
parse_package_spec("owner/repo:subpath")
    → fetch_package()            # GitHub tarball DL → tempdir展開
    → detect_from_dir()          # ファイル構造からターゲット型を判定（1件以外はエラー）
    → install_targets()          # 型ごとにdestを決定してファイルをコピー
    → copm.json / copm.lock更新  # installed_filesを記録
```

## ターゲット型とインストール先

| ターゲット型 | 検出条件 | ローカルインストール先 |
|---|---|---|
| `skill` | `SKILL.md` あり | `tools`設定に依存（下記） |
| `copilot-instructions` | `copilot-instructions.md` あり | `.github/copilot-instructions.md` |
| `copilot-custom-instructions` | `*.instructions.md` あり | `.github/instructions/` |
| `copilot-agents` | `*.agent.md` あり | `.github/agents/` |
| `copilot-prompts` | `*.prompt.md` あり | `.github/prompts/` |
| `claude-command` | `*.md`（コマンド文脈） | `.claude/commands/` |

`skill`のインストール先：
- `tools=["copilot"]` → `.github/skills/<name>/`（global: `~/.copilot/skills/<name>/`）
- `tools=["claude"]` → `.claude/skills/<name>/`（global: `~/.claude/skills/<name>/`）

## 新しいターゲット型の追加手順

1. `src/paths.rs` にパス関数を追加
2. `src/installer/copilot.rs` に `install_*` / `uninstall_*` / `list_*` を実装
3. `src/manifest/package_manifest.rs` の `classify_dir()` に検出ロジックを追加
4. `src/installer/mod.rs` の `install_target()` / `uninstall_targets()` にmatch追加
5. `src/commands/list.rs` に一覧表示を追加
6. `tests/integration_test.rs` にテストを追加

## 依存クレート

| クレート | 用途 |
|---|---|
| `clap` (derive) | CLI引数パース |
| `serde` + `serde_json` | JSON シリアライズ |
| `tokio` + `reqwest` | 非同期HTTP（GitHub API） |
| `dirs` | ホームディレクトリ解決 |
| `thiserror` | エラー型定義 |
| `tempfile` | 一時ディレクトリ |
| `walkdir` | 再帰ディレクトリコピー |
| `sha2` + `hex` | integrity hash（SHA-256） |
| `flate2` + `tar` | tarball展開 |

## 注意事項

- `copm-package.json` はサポートしない（インストール元リポジトリに設定ファイル不要）
- `error.rs` の `AmbiguousTargets` は `source` という名前のフィールドを持てない（thiserror予約語）→ `pkg` を使う
- インストール先のパスはすべて `src/paths.rs` に集約する（命令的なパス文字列をコード中に書かない）
