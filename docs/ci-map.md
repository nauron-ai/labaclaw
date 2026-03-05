# CI Workflow Map

This repository now uses a minimal CI/CD workflow surface.

## Active Workflows

- `.github/workflows/ci-run.yml`
  - Pull request and push quality gate.
  - Rust lint/test/build smoke and docs quality checks.
  - Merge gate: `CI Required Gate`.

- `.github/workflows/release-build.yml`
  - Production binary build on `main`, `v*` tags, and manual dispatch.
  - Publishes `zeroclaw-linux-amd64` workflow artifact (`zeroclaw` + `.sha256`).

- `.github/workflows/pub-release.yml`
  - Verification builds (manual/scheduled) and release publish on stable tags `vX.Y.Z`.
  - Enforces release trigger and artifact contract guards.

## Trigger Map

- `ci-run.yml`: PR/push merge-quality checks.
- `release-build.yml`: push to `main`, push tags `v*`, manual dispatch.
- `pub-release.yml`: push tags `v*`, manual dispatch, weekly verify schedule.

## Operational Notes

- Docker publish is intentionally disabled and not part of CI/CD policy.
- Release publish authorization is controlled by `RELEASE_AUTHORIZED_ACTORS`.
- Keep required branch protection checks aligned with this reduced set.
