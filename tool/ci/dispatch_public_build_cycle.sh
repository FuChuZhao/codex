#!/usr/bin/env bash
set -euo pipefail

# Minimal public-build closed loop:
# 1) gh workflow run
# 2) gh run watch --exit-status
# 3) gh run download
# 4) gh api DELETE artifacts

usage() {
  cat <<USAGE
Usage:
  $(basename "$0") \\
    --repo <BUILD_REPO> \\
    --workflow <WORKFLOW_FILE_OR_NAME> \\
    --source-repo <SOURCE_REPO> \\
    --source-ref <SOURCE_REF> \\
    [--ref <WORKFLOW_REF>] \\
    [--artifact-name <ARTIFACT_NAME>] \\
    [--build-script <BUILD_SCRIPT>] \\
    [--artifact-path <ARTIFACT_PATH>] \\
    [--download-artifact <true|false>] \\
    [--delete-artifacts-after-download <true|false>] \\
    [--out-dir <OUT_DIR>]

Required interface mapping:
  Repo                          -> --repo
  Workflow                      -> --workflow
  Ref                           -> --ref
  Inputs(source_ref/...)        -> --source-ref + --artifact-name + --build-script + --artifact-path
  DownloadArtifact              -> --download-artifact
  DeleteArtifactsAfterDownload  -> --delete-artifacts-after-download
  OutDir                        -> --out-dir
USAGE
}

normalize_bool() {
  local value="${1,,}"
  case "$value" in
    true|1|yes|y) echo "true" ;;
    false|0|no|n) echo "false" ;;
    *)
      echo "invalid boolean value: $1" >&2
      exit 2
      ;;
  esac
}

BUILD_REPO=""
WORKFLOW=""
WORKFLOW_REF="main"
SOURCE_REPO=""
SOURCE_REF=""
ARTIFACT_NAME="codex-build"
BUILD_SCRIPT="tool/ci/build_codex_cli_release.sh"
ARTIFACT_PATH="out"
DOWNLOAD_ARTIFACT="true"
DELETE_ARTIFACTS_AFTER_DOWNLOAD="true"
OUT_DIR="./out/cloud-artifacts"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      BUILD_REPO="${2:-}"
      shift 2
      ;;
    --workflow)
      WORKFLOW="${2:-}"
      shift 2
      ;;
    --ref)
      WORKFLOW_REF="${2:-}"
      shift 2
      ;;
    --source-repo)
      SOURCE_REPO="${2:-}"
      shift 2
      ;;
    --source-ref)
      SOURCE_REF="${2:-}"
      shift 2
      ;;
    --artifact-name)
      ARTIFACT_NAME="${2:-}"
      shift 2
      ;;
    --build-script)
      BUILD_SCRIPT="${2:-}"
      shift 2
      ;;
    --artifact-path)
      ARTIFACT_PATH="${2:-}"
      shift 2
      ;;
    --download-artifact)
      DOWNLOAD_ARTIFACT="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --delete-artifacts-after-download)
      DELETE_ARTIFACTS_AFTER_DOWNLOAD="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --out-dir)
      OUT_DIR="${2:-}"
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

if [[ -z "$BUILD_REPO" || -z "$WORKFLOW" || -z "$SOURCE_REPO" || -z "$SOURCE_REF" ]]; then
  echo "missing required arguments" >&2
  usage
  exit 2
fi

if ! [[ "$BUILD_REPO" =~ ^[^/]+/[^/]+$ ]]; then
  echo "--repo must be in OWNER/REPO format" >&2
  exit 2
fi

start_epoch="$(date -u +%s)"

echo "Dispatching workflow '${WORKFLOW}' on ${BUILD_REPO}@${WORKFLOW_REF} ..."
gh workflow run "$WORKFLOW" \
  -R "$BUILD_REPO" \
  --ref "$WORKFLOW_REF" \
  -f source_repo="$SOURCE_REPO" \
  -f source_ref="$SOURCE_REF" \
  -f artifact_name="$ARTIFACT_NAME" \
  -f build_script="$BUILD_SCRIPT" \
  -f artifact_path="$ARTIFACT_PATH"

echo "Resolving newly triggered run id ..."
run_id=""
for _ in $(seq 1 40); do
  runs_json="$(gh run list -R "$BUILD_REPO" --workflow "$WORKFLOW" --event workflow_dispatch --limit 30 --json databaseId,createdAt,status,conclusion,headBranch)"
  run_id="$(RUNS_JSON="$runs_json" python3 - "$start_epoch" <<'PY'
import datetime as dt
import json
import os
import sys

start_epoch = int(sys.argv[1])
try:
    rows = json.loads(os.environ.get("RUNS_JSON", "[]"))
except json.JSONDecodeError:
    rows = []

best = None
for row in rows:
    created = row.get("createdAt")
    if not created:
        continue
    try:
        ts = int(dt.datetime.fromisoformat(created.replace("Z", "+00:00")).timestamp())
    except ValueError:
        continue
    if ts < start_epoch - 15:
        continue
    if best is None or ts > best[0]:
        best = (ts, row.get("databaseId"))

print(best[1] if best else "")
PY
)"
  if [[ -n "$run_id" ]]; then
    break
  fi
  sleep 3
done

if [[ -z "$run_id" ]]; then
  echo "failed to resolve workflow run id" >&2
  exit 1
fi

echo "Watching run ${run_id} ..."
gh run watch "$run_id" -R "$BUILD_REPO" --exit-status

if [[ "$DOWNLOAD_ARTIFACT" == "true" ]]; then
  mkdir -p "$OUT_DIR"
  echo "Downloading artifact '${ARTIFACT_NAME}' to ${OUT_DIR} ..."
  gh run download "$run_id" -R "$BUILD_REPO" -n "$ARTIFACT_NAME" -D "$OUT_DIR"
fi

if [[ "$DELETE_ARTIFACTS_AFTER_DOWNLOAD" == "true" ]]; then
  echo "Deleting artifacts for run ${run_id} ..."
  mapfile -t artifact_ids < <(gh api -R "$BUILD_REPO" "/repos/${BUILD_REPO}/actions/runs/${run_id}/artifacts" --jq '.artifacts[].id')
  for artifact_id in "${artifact_ids[@]:-}"; do
    if [[ -n "$artifact_id" ]]; then
      gh api -R "$BUILD_REPO" -X DELETE "/repos/${BUILD_REPO}/actions/artifacts/${artifact_id}" >/dev/null
      echo "deleted artifact ${artifact_id}"
    fi
  done
fi

echo "build cycle completed"
echo "run_id=${run_id}"
