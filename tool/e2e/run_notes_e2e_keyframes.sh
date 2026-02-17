#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage:
  $(basename "$0") --workspace <SOURCE_ROOT> --codex-bin <CODEX_BIN> --out-dir <OUT_DIR> [--seed-port <PORT>]
USAGE
}

WORKSPACE=""
CODEX_BIN=""
OUT_DIR=""
SEED_PORT="8765"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --workspace)
      WORKSPACE="${2:-}"
      shift 2
      ;;
    --codex-bin)
      CODEX_BIN="${2:-}"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="${2:-}"
      shift 2
      ;;
    --seed-port)
      SEED_PORT="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$WORKSPACE" || -z "$CODEX_BIN" || -z "$OUT_DIR" ]]; then
  echo "--workspace, --codex-bin and --out-dir are required" >&2
  usage
  exit 2
fi

if [[ ! -x "$CODEX_BIN" ]]; then
  echo "codex binary not found or not executable: $CODEX_BIN" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
KEYFRAME_DIR="${OUT_DIR}/keyframes"
SEED_WORKSPACE="${WORKSPACE}/e2e-workspace"
PLAN_FILE="${OUT_DIR}/keyframes.plan"
SERVER_LOG="${OUT_DIR}/seed-server.log"

python3_cmd() {
  python3 "$@"
}

seed_server_pid=""
cleanup() {
  if [[ -n "$seed_server_pid" ]]; then
    kill "$seed_server_pid" >/dev/null 2>&1 || true
    wait "$seed_server_pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

python3_cmd "${WORKSPACE}/tool/e2e/mock_notes_seed_server.py" --host 127.0.0.1 --port "$SEED_PORT" > "$SERVER_LOG" 2>&1 &
seed_server_pid="$!"

for _ in $(seq 1 30); do
  if python3 - <<PY
import json
import urllib.request
url = "http://127.0.0.1:${SEED_PORT}/health"
with urllib.request.urlopen(url, timeout=2) as resp:  # noqa: S310
    payload = json.loads(resp.read().decode("utf-8"))
assert payload.get("ok") is True
PY
  then
    break
  fi
  sleep 1
done

python3_cmd "${WORKSPACE}/tool/e2e/seed_workspace_from_mock.py" \
  --workspace "$SEED_WORKSPACE" \
  --seed-url "http://127.0.0.1:${SEED_PORT}/seed" \
  --replace > "${OUT_DIR}/seed-result.json"

cat > "$PLAN_FILE" <<PLAN
TYPE ${CODEX_BIN} note list --workspace ${SEED_WORKSPACE}
KEY Enter
WAIT 1
CAPTURE note-list

TYPE ${CODEX_BIN} search rollback --workspace ${SEED_WORKSPACE}
KEY Enter
WAIT 1
CAPTURE search

TYPE ${CODEX_BIN} snapshot resume --workspace ${SEED_WORKSPACE} --snapshot s_seed_1
KEY Enter
WAIT 1
CAPTURE snapshot-resume
PLAN

"${WORKSPACE}/tool/e2e/capture_fullscreen_keyframes.sh" \
  --workspace "$WORKSPACE" \
  --command "bash --noprofile --norc" \
  --out-dir "$KEYFRAME_DIR" \
  --plan "$PLAN_FILE" \
  --width 180 \
  --height 56

echo "e2e keyframes completed: ${KEYFRAME_DIR}"
