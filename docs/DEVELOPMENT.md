# 開発ガイド

## セットアップ

```bash
git clone https://github.com/ya-uhs/copm
cd copm
cargo build
```

## テスト

```bash
cargo test
```

テストはすべて `tests/integration_test.rs` に書く（ライブラリのユニットテストは不要）。

## アーキテクチャ

```
src/
├── main.rs                         # エントリポイント (tokio::main)
├── lib.rs                          # モジュールエクスポート
├── cli/
│   └── args.rs                     # clap derive によるCLI定義
├── commands/
│   ├── mod.rs                      # Command enum → 各コマンドへのディスパッチ
│   ├── init.rs                     # copm init（tools選択プロンプト → copm.json生成）
│   ├── install.rs                  # copm install（fetch → detect → install → lock更新）
│   ├── uninstall.rs                # copm uninstall（installed_filesベースで削除）
│   └── list.rs                     # copm list（全ターゲット型のインストール先を走査）
├── config/
│   ├── copm_json.rs                # copm.json（tools / dependencies / sub_path）
│   └── lock.rs                     # copm.lock（targets / installed_files）
├── manifest/
│   └── package_manifest.rs         # detect_from_dir()：ファイル構造からターゲット型を判定
├── fetcher/
│   └── git.rs                      # parse_package_spec() / fetch_package()（tarball or clone）
├── installer/
│   ├── mod.rs                      # install_targets() / uninstall_targets()：型別ディスパッチ
│   ├── copilot.rs                  # 全インストーラ本体（skill / agents / prompts / instructions）
│   └── claude_plugin.rs            # 旧 claude-plugin 型（後方互換のみ）
├── paths.rs                        # 全インストール先パスの定義
└── error.rs                        # CopmError (thiserror)
```

## データフロー

### install コマンド

```
ユーザ入力: copm install owner/repo:subpath
    │
    ▼
fetcher::git::parse_package_spec()
    → (owner, repo, Option<subpath>)
    │
    ▼
fetcher::git::fetch_package()
    ├── GitHub tarball API (https://api.github.com/repos/{owner}/{repo}/tarball/HEAD)
    └── fallback: git clone --depth 1
    │
    ▼
manifest::PackageManifest::detect_from_dir(dir, sub_path, source)
    ├── subpath指定あり → そのディレクトリのみスキャン → 1ターゲット型に確定
    └── subpath指定なし → ルートをスキャン
        ├── 1件 → インストール実行
        ├── 0件 → NoTargetsDetected エラー
        └── 複数件 → AmbiguousTargets エラー（候補一覧を表示）
    │
    ▼
installer::install_targets(source_dir, manifest, name, tools, global)
    ├── "copilot-instructions"       → .github/copilot-instructions.md
    ├── "copilot-custom-instructions"→ .github/instructions/*.instructions.md
    ├── "copilot-agents"             → .github/agents/*.agent.md
    ├── "copilot-prompts"            → .github/prompts/*.prompt.md
    ├── "skill" (tools=copilot)      → .github/skills/<name>/
    ├── "skill" (tools=claude)       → .claude/skills/<name>/
    └── "claude-command"             → .claude/commands/*.md
    │
    ▼
copm.json + copm.lock 更新
    └── installed_files に実インストールパスを記録（アンインストール時に使用）
```

### uninstall コマンド

```
copm.lock から installed_files を取得
    └── ファイル/ディレクトリを削除
    └── installed_files が空の場合は target_types ベースのレガシー削除にフォールバック
```

## ターゲット型の検出ロジック

`src/manifest/package_manifest.rs` の `classify_dir()` が判定する。

| 検出条件（対象ディレクトリ内） | ターゲット型 |
|---|---|
| `SKILL.md` が存在する | `skill` |
| `copilot-instructions.md` が存在する | `copilot-instructions` |
| `*.agent.md` を含む | `copilot-agents` |
| `*.prompt.md` を含む | `copilot-prompts` |
| `*.instructions.md` を含む | `copilot-custom-instructions` |
| サブディレクトリに `SKILL.md` を含む | `skill`（コレクション） |

subpath指定なしの場合：ルート直下のファイル → ルート直下の各サブディレクトリの順でスキャンし、複数のターゲット型が見つかれば `AmbiguousTargets` エラーとなる。

## 新しいターゲット型の追加手順

1. **`src/paths.rs`** にインストール先パス関数を追加
2. **`src/installer/copilot.rs`** に `install_*()` / `uninstall_*()` / `list_*()` を実装
3. **`src/manifest/package_manifest.rs`** の `classify_dir()` に検出ロジックを追加
4. **`src/installer/mod.rs`** の `install_target()` / `uninstall_targets()` のmatchに追加
5. **`src/commands/list.rs`** に一覧表示を追加
6. **`tests/integration_test.rs`** にテストを追加

## 設定ファイル仕様

### copm.json

```json
{
  "tools": ["copilot"],
  "dependencies": {
    "humanizer": {
      "source": "blader/humanizer",
      "version": "0.0.0"
    },
    "awesome-agents": {
      "source": "github/awesome-copilot",
      "sub_path": "agents",
      "version": "0.0.0"
    }
  }
}
```

- `tools`: `"copilot"` / `"claude"` / 両方。スキルのインストール先を決定する。デフォルト `["copilot"]`
- `sub_path`: `owner/repo:subpath` の `:subpath` 部分。`copm install -g` で復元時に使用

### copm.lock

```json
{
  "version": 1,
  "packages": [
    {
      "name": "humanizer",
      "version": "0.0.0",
      "source": { "type": "github", "repo": "blader/humanizer", "sub_path": null },
      "integrity": "sha256-...",
      "targets": ["skill"],
      "installed_files": [".github/skills/humanizer"]
    }
  ]
}
```

- `installed_files`: アンインストール時に削除するパスの一覧（ファイルまたはディレクトリ）
- `targets`: ターゲット型名の一覧（`installed_files` が空の場合のレガシーフォールバック用）

## 依存クレート

| クレート | 用途 |
|---|---|
| `clap` (derive) | CLI引数パース |
| `serde` + `serde_json` | JSON シリアライズ/デシリアライズ |
| `tokio` + `reqwest` | 非同期HTTP（GitHub API） |
| `dirs` | ホームディレクトリ解決（`~`） |
| `thiserror` | エラー型定義 |
| `tempfile` | 一時ディレクトリ（tarball展開用） |
| `walkdir` | 再帰的ディレクトリコピー |
| `sha2` + `hex` | integrity hash（SHA-256） |
| `flate2` + `tar` | tarball展開 |

`anyhow` は依存に残っているが現在未使用。

## エラー型

`CopmError` (`src/error.rs`) に全バリアントが定義されている。

注意: `AmbiguousTargets` は struct variant で `pkg` / `targets` フィールドを持つ。
`source` という名前は thiserror の予約フィールド（エラーの原因チェーン）のため使用不可。

## ロードマップ

- **v0.1.0**（現在）: install / uninstall / list / init、Copilot + Claude Code対応、`owner/repo:subpath` 形式
- **v0.2.0**: `copm update`、`copm search`
- **v0.3.0**: レジストリ対応、バージョン範囲指定
