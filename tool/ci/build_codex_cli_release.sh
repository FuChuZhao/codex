#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/out"
BIN_OUT_DIR="${OUT_DIR}/bin"
META_OUT_DIR="${OUT_DIR}/meta"

mkdir -p "${BIN_OUT_DIR}" "${META_OUT_DIR}"

pushd "${ROOT_DIR}/codex-rs" >/dev/null
# Build in cloud without forcing a lockfile rewrite failure when source_ref carries
# dependency edits that are not yet reflected in Cargo.lock.
cargo build -p codex-cli --release
popd >/dev/null

BIN_PATH="${ROOT_DIR}/codex-rs/target/release/codex"
if [[ ! -x "${BIN_PATH}" ]]; then
  echo "expected built binary not found: ${BIN_PATH}" >&2
  exit 1
fi

cp "${BIN_PATH}" "${BIN_OUT_DIR}/codex"
sha256sum "${BIN_OUT_DIR}/codex" > "${META_OUT_DIR}/codex.sha256"

{
  echo "built_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "source_repo=$(git -C "${ROOT_DIR}" remote get-url origin 2>/dev/null || echo unknown)"
  echo "source_commit=$(git -C "${ROOT_DIR}" rev-parse --short HEAD 2>/dev/null || echo unknown)"
} > "${META_OUT_DIR}/build-info.txt"

echo "build completed: ${BIN_OUT_DIR}/codex"
