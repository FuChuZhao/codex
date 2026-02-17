# E2E Keyframe Capture Helpers

This folder contains scripts for cloud/local deterministic keyframe capture:

- `mock_notes_seed_server.py`: serves seed data at `/seed`
- `seed_workspace_from_mock.py`: writes seed records into `<workspace>/.codex-notes`
- `capture_fullscreen_keyframes.sh`: records tmux fullscreen frames (`.ansi` + `.txt`)
- `run_notes_e2e_keyframes.sh`: starts mock server, seeds workspace, captures keyframes
- `sample_keyframes.plan`: example scripted interaction plan

## Quick run

```bash
tool/e2e/run_notes_e2e_keyframes.sh \
  --workspace "$(pwd)" \
  --codex-bin "$(pwd)/out/bin/codex" \
  --out-dir "$(pwd)/out/e2e"
```
