# CI Cloud Build Helpers

- `build_codex_cli_release.sh`
  - SOURCE_REPO build entry script. Produces `out/bin/codex` and metadata under `out/meta`.
- `dispatch_public_build_cycle.sh`
  - `gh` closed loop: dispatch workflow -> wait -> download artifact -> delete artifacts.
- `public-build-workflow.template.yml`
  - Workflow template for BUILD_REPO (`workflow_dispatch` only).

## Closed-loop example

```bash
tool/ci/dispatch_public_build_cycle.sh \
  --repo OWNER/BUILD_REPO \
  --workflow private-source-cloud-build.yml \
  --ref main \
  --source-repo OWNER/SOURCE_REPO \
  --source-ref main \
  --artifact-name codex-cloud-build \
  --build-script tool/ci/build_codex_cli_release.sh \
  --artifact-path out \
  --download-artifact true \
  --delete-artifacts-after-download true \
  --out-dir ./out/cloud-artifacts
```
