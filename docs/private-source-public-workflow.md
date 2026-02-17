# Private SOURCE_REPO + Public BUILD_REPO Workflow

This document implements the minimal-exposure workflow pattern:

- SOURCE_REPO (private): source code + build/e2e scripts
- BUILD_REPO (public): workflow-only repo that clones SOURCE_REPO over SSH and uploads short-lived artifacts

## Security constraints

- BUILD_REPO workflow trigger: `workflow_dispatch` only
- BUILD_REPO permissions: `contents: read`
- SSH key policy:
  - one read-only deploy key per SOURCE_REPO
  - public key added to SOURCE_REPO Deploy Keys (read-only)
  - private key stored as BUILD_REPO secret `SOURCE_REPO_SSH_KEY`
- Artifact retention: `1` day
- Build logs must not print secret values

## Files in this repo

- Workflow (for BUILD_REPO):
  - `.github/workflows/private-source-cloud-build.yml`
- Workflow template copy:
  - `tool/ci/public-build-workflow.template.yml`
- SOURCE_REPO cloud build entry script:
  - `tool/ci/build_codex_cli_release.sh`
- SOURCE_REPO dispatch/watch/download/delete loop:
  - `tool/ci/dispatch_public_build_cycle.sh`

## Trigger -> wait -> download -> delete (gh closed loop)

```bash
tool/ci/dispatch_public_build_cycle.sh \
  --repo <BUILD_REPO_OWNER/BUILD_REPO_NAME> \
  --workflow private-source-cloud-build.yml \
  --ref main \
  --source-repo <SOURCE_REPO_OWNER/SOURCE_REPO_NAME> \
  --source-ref <branch-or-commit> \
  --artifact-name codex-cloud-build \
  --build-script tool/ci/build_codex_cli_release.sh \
  --artifact-path out \
  --download-artifact true \
  --delete-artifacts-after-download true \
  --out-dir ./out/cloud-artifacts
```

## E2E keyframe capture in cloud

The workflow can run the e2e keyframe job (`run_e2e=true`) after build. It uses:

- `tool/e2e/mock_notes_seed_server.py` (local mock data server)
- `tool/e2e/seed_workspace_from_mock.py` (populate `.codex-notes`)
- `tool/e2e/run_notes_e2e_keyframes.sh` (orchestration)
- `tool/e2e/capture_fullscreen_keyframes.sh` (fullscreen frame capture with tmux)

Captured keyframes are stored under `out/e2e/keyframes` and included in the uploaded artifact.
