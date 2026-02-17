#!/usr/bin/env python3
from __future__ import annotations

import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
from typing import Any


DEFAULT_SEED: dict[str, Any] = {
    "conversations": [
        {
            "id": "c_seed_main",
            "title": "seed-main",
            "created_at": 1735689600,
            "updated_at": 1735689660,
            "root_message_id": "m_seed_1",
        }
    ],
    "messages": [
        {
            "id": "m_seed_1",
            "conversation_id": "c_seed_main",
            "parent_id": None,
            "role": "user",
            "content": "Need rollout checklist for codex note workflow",
            "created_at": 1735689601,
        },
        {
            "id": "m_seed_2",
            "conversation_id": "c_seed_main",
            "parent_id": "m_seed_1",
            "role": "assistant",
            "content": "Create branch, snapshot, then export for handoff",
            "created_at": 1735689660,
        },
    ],
    "notes": [
        {
            "id": "n_seed_1",
            "conversation_id": "c_seed_main",
            "message_id": "m_seed_2",
            "title": "Rollout notes",
            "body": "Validate artifact retention is 1 day and delete after download.",
            "tags": ["risk", "release"],
            "status": "open",
            "priority": "p1",
            "created_at": 1735689661,
            "updated_at": 1735689661,
            "repo_ctx": {
                "repo_path": "/tmp/source",
                "git_branch": "feature/notes",
                "git_commit": "seed123",
                "related_files": ["tool/ci/dispatch_public_build_cycle.sh"],
            },
        }
    ],
    "branches": [
        {
            "id": "b_seed_1",
            "source_conversation_id": "c_seed_main",
            "source_message_id": "m_seed_2",
            "new_conversation_id": "c_seed_branch",
            "created_at": 1735689700,
        }
    ],
    "snapshots": [
        {
            "id": "s_seed_1",
            "conversation_id": "c_seed_main",
            "summary": "Prepared cloud build + e2e capture checklist.",
            "todo": ["run workflow_dispatch", "download artifact", "delete artifacts"],
            "risks": ["artifact retention too long"],
            "repo_ctx": {
                "repo_path": "/tmp/source",
                "git_branch": "feature/notes",
                "git_commit": "seed123",
                "related_files": [".github/workflows/private-source-cloud-build.yml"],
            },
            "created_at": 1735689800,
        }
    ],
}


def load_seed(seed_file: Path | None) -> dict[str, Any]:
    if seed_file is None:
        return DEFAULT_SEED
    raw = seed_file.read_text(encoding="utf-8")
    data = json.loads(raw)
    if not isinstance(data, dict):
        raise ValueError("seed file must be a JSON object")
    return data


class SeedHandler(BaseHTTPRequestHandler):
    seed_payload: dict[str, Any] = {}

    def _send_json(self, status: int, payload: dict[str, Any]) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt: str, *args: object) -> None:  # noqa: A003
        return

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/health":
            self._send_json(200, {"ok": True})
            return
        if self.path == "/seed":
            self._send_json(200, self.seed_payload)
            return
        self._send_json(404, {"error": "not_found"})


def main() -> int:
    parser = argparse.ArgumentParser(description="Serve deterministic codex note seed data")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--seed-file", type=Path)
    args = parser.parse_args()

    seed_payload = load_seed(args.seed_file)
    SeedHandler.seed_payload = seed_payload

    server = ThreadingHTTPServer((args.host, args.port), SeedHandler)
    print(f"seed server listening on http://{args.host}:{args.port}", flush=True)
    server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
