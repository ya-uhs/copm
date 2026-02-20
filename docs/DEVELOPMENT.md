# 開発ガイド

## セットアップ

```bash
git clone <repo>
cd copm
cargo build
```

## テスト

```bash
cargo test
```

## アーキテクチャ

```
src/
├── main.rs                         # エントリポイント (tokio::main)
├── lib.rs                          # モジュールエクスポート
├── cli/
│   └── args.rs                     # clap derive によるCLI定義
├── commands/
│   ├── mod.rs                      # Command enum → 各コマンドへのディスパッチ
│   ├── init.rs                     # copm init
│   ├── install.rs                  # copm install (非同期)
│   ├── uninstall.rs                # copm uninstall
│   └── list.rs                     # copm list
├── config/
│   ├── copm_json.rs                # copm.json の読み書き
│   └── lock.rs                     # copm.lock の読み書き
├── manifest/
│   └── package_manifest.rs         # copm-package.json のパース + 自動検出
├── fetcher/
│   └── git.rs                      # GitHub tarball DL / git clone fallback
├── installer/
│   ├── mod.rs                      # target type別ディスパッチ
│   ├── claude_plugin.rs            # Claude Code プラグインインストーラ
│   └── copilot.rs                  # Copilot instructions インストーラ
├── paths.rs                        # 全パス定義の集約
└── error.rs                        # CopmError (thiserror)
```

## データフロー

### install コマンド

```
ユーザ入力: copm install user/repo
    │
    ▼
fetcher::git::parse_package_spec("user/repo")
    │
    ▼
fetcher::git::fetch_package()
    ├── GitHub tarball API (https://api.github.com/repos/{user}/{repo}/tarball/HEAD)
    └── fallback: git clone --depth 1
    │
    ▼
manifest::PackageManifest::load_from_dir()
    ├── copm-package.json があればパース
    └── なければ自動検出 (.claude-plugin/, .github/copilot-instructions.md, etc.)
    │
    ▼
installer::install_targets()  ← target type ごとにディスパッチ
    ├── claude_plugin::install_plugin()   → .claude/plugins/{name}/
    ├── copilot::install_instructions()   → .github/copilot-instructions.md
    └── copilot::install_custom_instructions() → .github/instructions/
    │
    ▼
config 更新 (copm.json + copm.lock)
```

## 新しいターゲットの追加方法

1. **`src/installer/` に新モジュールを作成** (例: `src/installer/cursor.rs`)
   - `install_*()`, `uninstall_*()`, `list_*()` を実装

2. **`src/installer/mod.rs` のディスパッチに追加**
   - `install_target()` の match に新しい target_type を追加
   - `uninstall_targets()` にも追加

3. **`src/paths.rs` にパス定義を追加**

4. **`src/manifest/package_manifest.rs` に自動検出ロジックを追加** (任意)

5. **`src/commands/list.rs` に一覧表示を追加**

6. **テストを追加** (`tests/integration_test.rs`)

## 依存クレート

| クレート | 用途 |
|---------|------|
| `clap` (derive) | CLI引数パース |
| `serde` + `serde_json` | JSON シリアライズ/デシリアライズ |
| `tokio` + `reqwest` | 非同期HTTP (GitHub API) |
| `dirs` | `~` ホームディレクトリ解決 |
| `thiserror` | エラー型定義 |
| `anyhow` | エラーハンドリングユーティリティ |
| `tempfile` | 一時ディレクトリ (ダウンロード用) |
| `walkdir` | 再帰的ディレクトリ走査 |
| `sha2` + `hex` | integrity hash (SHA-256) |
| `flate2` + `tar` | tarball展開 |

## エラー型

`CopmError` (`src/error.rs`) に全エラーバリアントが定義されている。
新しいエラーを追加する場合は `#[error("...")]` アトリビュートと共にバリアントを追加する。

## ロードマップ

- **v0.1.0** (現在): install / uninstall / list / init、Claude Code + Copilot対応
- **v0.2.0**: `copm update`, `copm search`, CLAUDE.md対応
- **v0.3.0**: レジストリ対応、バージョン範囲指定
