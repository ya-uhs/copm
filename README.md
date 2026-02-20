# copm

AI Coding Assistant向けパッケージマネージャ。

GitHub CopilotやClaude Codeのスキル・instructions・agentsを、1コマンドでGitHubからインストールできる。

## インストール

```bash
cargo install --path .
```

## 使い方

### プロジェクトの初期化

```bash
copm init
```

使用ツール（`copilot` / `claude` / `both`）を選択すると `copm.json` が生成される。

### パッケージのインストール

```bash
# シンプルなリポジトリ（ルートにSKILL.mdなど）
copm install blader/humanizer

# コレクション系リポジトリのサブパスを指定
copm install github/awesome-copilot:agents
copm install github/awesome-copilot:skills/planning
copm install github/awesome-copilot:instructions

# グローバルインストール
copm install -g blader/humanizer

# copm.json の依存をすべてインストール
copm install
```

ターゲット型はリポジトリの内容から自動検出される。複数の型が混在する場合は `:subpath` で絞り込む。

```
Error: Multiple targets detected in github/awesome-copilot:
  agents        (copilot-agents)
  instructions  (copilot-custom-instructions)
  prompts       (copilot-prompts)
  skills        (skill)
Use: copm install github/awesome-copilot:<subpath>
```

### アンインストール

```bash
copm uninstall humanizer
copm uninstall -g humanizer
```

### インストール済み一覧

```bash
copm list
copm list -g
```

---

## ファイルのインストール先

### ローカル（プロジェクト単位）

| ターゲット型 | インストール先 |
|---|---|
| `copilot-instructions` | `.github/copilot-instructions.md` |
| `copilot-custom-instructions` | `.github/instructions/*.instructions.md` |
| `copilot-agents` | `.github/agents/*.agent.md` |
| `copilot-prompts` | `.github/prompts/*.prompt.md` |
| `skill`（tools=copilot） | `.github/skills/<name>/` |
| `skill`（tools=claude） | `.claude/skills/<name>/` |
| `skill`（tools=both） | 両方 |
| `claude-command` | `.claude/commands/*.md` |

### グローバル

| ターゲット型 | インストール先 |
|---|---|
| `copilot-custom-instructions` | `~/.copilot/instructions/*.instructions.md` |
| `skill`（tools=copilot） | `~/.copilot/skills/<name>/` |
| `skill`（tools=claude） | `~/.claude/skills/<name>/` |
| `claude-command` | `~/.claude/commands/*.md` |

> `copilot-instructions` / `copilot-agents` / `copilot-prompts` はグローバル配置先が未定義のため、`-g` 指定時はスキップされる。

---

## 自動検出ロジック

インストール元リポジトリに `copm-package.json` は不要。
ファイル構造から以下のルールで自動判定する。

### サブパス指定あり（`owner/repo:subpath`）

指定ディレクトリのみをスキャンし、ファイルのサフィックスで1つのターゲット型に確定する。

| 検出条件 | ターゲット型 |
|---|---|
| `SKILL.md` が存在する | `skill` |
| `copilot-instructions.md` が存在する | `copilot-instructions` |
| `*.agent.md` を含む | `copilot-agents` |
| `*.prompt.md` を含む | `copilot-prompts` |
| `*.instructions.md` を含む | `copilot-custom-instructions` |
| サブディレクトリに `SKILL.md` を含む | `skill`（コレクション） |

### サブパス指定なし（`owner/repo`）

ルートをスキャンし、検出数で分岐する。

- **1件** → インストール実行
- **0件** → エラー（`NoTargetsDetected`）
- **複数件** → エラー（`AmbiguousTargets`）、候補一覧を表示

スキャン対象：ルート直下のファイル、およびルート直下の各サブディレクトリ。

---

## 設定ファイル

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

| フィールド | 説明 | デフォルト |
|---|---|---|
| `tools` | 使用ツール（`"copilot"` / `"claude"` / 両方） | `["copilot"]` |
| `dependencies` | インストールするパッケージ一覧 | `{}` |
| `dependencies.*.source` | `owner/repo` 形式のGitHubリポジトリ | 必須 |
| `dependencies.*.sub_path` | リポジトリ内のサブパス | なし |
| `dependencies.*.version` | バージョン（現在は記録のみ） | `"0.0.0"` |

### copm.lock

インストール済みパッケージのバージョン・integrity・インストールファイル一覧を記録する。手動編集は不要。

```json
{
  "version": 1,
  "packages": [
    {
      "name": "humanizer",
      "version": "0.0.0",
      "source": { "type": "github", "repo": "blader/humanizer" },
      "integrity": "sha256-...",
      "targets": ["skill"],
      "installed_files": [".github/skills/humanizer"]
    }
  ]
}
```

`installed_files` に記録されたパスを使ってアンインストール時に正確に削除する。

---

## パッケージの作り方

インストール元リポジトリ側に設定ファイルは一切不要。
ファイル構造だけで自動検出される。

### 単一スキル（SKILL.md）

```
my-skill/
└── SKILL.md       ← 必須
    prompt.md
    README.md
```

```bash
copm install yourname/my-skill
# → .github/skills/my-skill/ にコピー（tools=copilot の場合）
```

### カスタムインストラクション集

```
my-instructions/
├── react.instructions.md
├── testing.instructions.md
└── README.md
```

```bash
copm install yourname/my-instructions
# → .github/instructions/*.instructions.md にコピー
```

### Agent集

```
my-agents/
├── architect.agent.md
├── reviewer.agent.md
└── README.md
```

```bash
copm install yourname/my-agents
# → .github/agents/*.agent.md にコピー
```

### コレクション系リポジトリ（複数型が混在）

```
awesome-copilot/
├── agents/
│   ├── architect.agent.md
│   └── reviewer.agent.md
├── instructions/
│   └── react.instructions.md
├── prompts/
│   └── code-review.prompt.md
└── skills/
    ├── planning/
    │   └── SKILL.md
    └── writing/
        └── SKILL.md
```

```bash
# サブパスを指定してインストール
copm install github/awesome-copilot:agents
copm install github/awesome-copilot:instructions
copm install github/awesome-copilot:prompts
copm install github/awesome-copilot:skills/planning

# サブパス省略 → AmbiguousTargets エラー
copm install github/awesome-copilot
```

## License

MIT
