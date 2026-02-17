#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage:
  $(basename "$0") \
    --workspace <PATH> \
    [--command <CMD>] \
    --out-dir <PATH> \
    [--plan <FILE>] \
    [--session-name <NAME>] \
    [--width <COLS>] \
    [--height <ROWS>] \
    [--keep-session <true|false>]

Plan syntax (one action per line):
  WAIT <seconds>
  TYPE <literal text>
  KEY <tmux-key>
  CAPTURE <label>

Examples:
  TYPE codex note list --workspace /tmp/ws
  KEY Enter
  WAIT 1
  CAPTURE after-note-list
USAGE
}

normalize_bool() {
  case "${1,,}" in
    true|1|yes|y) echo "true" ;;
    false|0|no|n) echo "false" ;;
    *)
      echo "invalid boolean value: $1" >&2
      exit 2
      ;;
  esac
}

WORKSPACE=""
COMMAND="bash --noprofile --norc"
OUT_DIR=""
PLAN=""
SESSION_NAME="codex-keyframes-$$"
WIDTH="160"
HEIGHT="48"
KEEP_SESSION="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --workspace)
      WORKSPACE="${2:-}"
      shift 2
      ;;
    --command)
      COMMAND="${2:-}"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="${2:-}"
      shift 2
      ;;
    --plan)
      PLAN="${2:-}"
      shift 2
      ;;
    --session-name)
      SESSION_NAME="${2:-}"
      shift 2
      ;;
    --width)
      WIDTH="${2:-}"
      shift 2
      ;;
    --height)
      HEIGHT="${2:-}"
      shift 2
      ;;
    --keep-session)
      KEEP_SESSION="$(normalize_bool "${2:-}")"
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

if [[ -z "$WORKSPACE" || -z "$OUT_DIR" ]]; then
  echo "--workspace and --out-dir are required" >&2
  usage
  exit 2
fi

if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux is required for fullscreen keyframe capture" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
manifest_path="${OUT_DIR}/manifest.jsonl"
: > "$manifest_path"

capture_frame() {
  local label="$1"
  local index="$2"
  local stamp
  stamp="$(date -u +%Y%m%dT%H%M%SZ)"
  local ansi_file="${OUT_DIR}/frame-$(printf '%03d' "$index")-${label}.ansi"
  local text_file="${OUT_DIR}/frame-$(printf '%03d' "$index")-${label}.txt"

  tmux capture-pane -ep -t "${SESSION_NAME}:0.0" > "$ansi_file"

  python3 - "$ansi_file" "$text_file" <<'PY'
import re
import sys
from pathlib import Path

src = Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
# Strip ANSI CSI sequences for plain text companion.
clean = re.sub(r"\x1B\[[0-?]*[ -/]*[@-~]", "", src)
Path(sys.argv[2]).write_text(clean, encoding="utf-8")
PY

  printf '{"index":%s,"label":"%s","timestamp":"%s","ansi":"%s","text":"%s"}\n' \
    "$index" "$label" "$stamp" "$(basename "$ansi_file")" "$(basename "$text_file")" >> "$manifest_path"
}

cleanup() {
  if [[ "$KEEP_SESSION" == "false" ]]; then
    tmux kill-session -t "$SESSION_NAME" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

runner_script="${OUT_DIR}/.tmux-runner.sh"
cat > "$runner_script" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$WORKSPACE"
exec bash -lc '$COMMAND'
EOF
chmod +x "$runner_script"

tmux new-session -d -s "$SESSION_NAME" -x "$WIDTH" -y "$HEIGHT" "$runner_script"
sleep 1

frame_index=0
capture_frame "boot" "$frame_index"
frame_index=$((frame_index + 1))

if [[ -n "$PLAN" ]]; then
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "$line" ]] && continue
    [[ "$line" =~ ^# ]] && continue

    action="${line%% *}"
    arg="${line#* }"
    if [[ "$line" == "$action" ]]; then
      arg=""
    fi

    case "$action" in
      WAIT)
        sleep "$arg"
        ;;
      TYPE)
        tmux send-keys -t "${SESSION_NAME}:0.0" -- "$arg"
        ;;
      KEY)
        tmux send-keys -t "${SESSION_NAME}:0.0" "$arg"
        ;;
      CAPTURE)
        capture_frame "$arg" "$frame_index"
        frame_index=$((frame_index + 1))
        continue
        ;;
      *)
        echo "unknown plan action: $action" >&2
        exit 2
        ;;
    esac

    label="$(echo "$action" | tr '[:upper:]' '[:lower:]')"
    capture_frame "$label" "$frame_index"
    frame_index=$((frame_index + 1))
  done < "$PLAN"
fi

capture_frame "final" "$frame_index"

echo "keyframes saved in: ${OUT_DIR}"
