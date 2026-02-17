#!/usr/bin/env python3
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import shutil
import urllib.request


def now_ts() -> int:
    return int(datetime.now(tz=timezone.utc).timestamp())


def fetch_seed(seed_url: str) -> dict:
    with urllib.request.urlopen(seed_url, timeout=10) as resp:  # noqa: S310
        if resp.status != 200:
            raise RuntimeError(f"seed endpoint returned {resp.status}")
        payload = json.loads(resp.read().decode("utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError("seed payload must be a JSON object")
    return payload


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Seed .codex-notes from a local mock server")
    parser.add_argument("--workspace", type=Path, required=True)
    parser.add_argument("--seed-url", default="http://127.0.0.1:8765/seed")
    parser.add_argument("--replace", action="store_true", help="delete existing .codex-notes before writing")
    args = parser.parse_args()

    workspace = args.workspace.resolve()
    store = workspace / ".codex-notes"
    if args.replace and store.exists():
        shutil.rmtree(store)

    payload = fetch_seed(args.seed_url)

    conversations = payload.get("conversations", [])
    messages = payload.get("messages", [])
    notes = payload.get("notes", [])
    branches = payload.get("branches", [])
    snapshots = payload.get("snapshots", [])

    for row in conversations:
        write_json(store / "conversations" / f"{row['id']}.json", row)
    for row in messages:
        write_json(store / "messages" / f"{row['id']}.json", row)
    for row in notes:
        write_json(store / "notes" / f"{row['id']}.json", row)
    for row in branches:
        write_json(store / "branches" / f"{row['id']}.json", row)
    for row in snapshots:
        write_json(store / "snapshots" / f"{row['id']}.json", row)

    summary = {
        "version": 1,
        "generated_at": now_ts(),
        "conversations": len(conversations),
        "messages": len(messages),
        "notes": len(notes),
        "branches": len(branches),
        "snapshots": len(snapshots),
    }
    write_json(store / "index.json", summary)

    print(json.dumps({"seeded": summary, "store": str(store)}, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
