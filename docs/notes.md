# Notes, Branches, and Snapshots

Codex CLI provides local note-management commands that store structured records inside your workspace.

## Storage layout

All commands write data under `<workspace>/.codex-notes/`:

```text
.codex-notes/
  conversations/
  messages/
  notes/
  branches/
  snapshots/
  exports/
  index.json
```

## Core commands

```bash
# conversation + message records
codex conversation create --title "main"
codex message add --conversation <conversation_id> --role user --content "Need rollback plan"

# notes
codex note add --title "Plan" --body "Use feature flag" --conversation <conversation_id> --tag risk --priority p1
codex note annotate --message <message_id> --body "Need load test evidence"
codex note list --status open --tag risk

# branch + snapshot
codex branch fork --message <message_id> --title "low-risk-path"
codex branch tree --conversation <conversation_id>
codex snapshot create --conversation <conversation_id> --from-latest
codex snapshot resume --snapshot <snapshot_id>

# search + export + index
codex search "rollback" --repo current
codex export conversation --id <conversation_id> --with-branches
codex index rebuild
```

## Filters

`codex note list` and `codex search` support:

- `--conversation <id>`
- `--tag <tag>`
- `--status <draft|open|blocked|done|archived>`
- `--repo current` or `--repo <absolute-repo-path>`

## Snapshots

Use `codex snapshot create --from-latest` to build a summary from recent messages and notes when you do not want to write one manually.
Use `codex snapshot resume --snapshot <id>` to print a compact context block for continuing work later.

## Export

`codex export conversation --id <conversation_id>` writes a Markdown export in `.codex-notes/exports/`.
Use `--with-branches` to include descendant branch conversations.
