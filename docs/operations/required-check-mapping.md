# Required Check Mapping

This document maps merge/release-critical checks to the remaining workflow set.

## Merge to `main`

| Required check name | Source workflow | Scope |
| --- | --- | --- |
| `CI Required Gate` | `.github/workflows/ci-run.yml` | core merge gate |

## Release

| Required check name | Source workflow | Scope |
| --- | --- | --- |
| `Build and Test (Linux x86_64)` | `.github/workflows/release-build.yml` | production binary build |
| `Verify Artifact Set` | `.github/workflows/pub-release.yml` | release artifact completeness |

## Notes

- Keep check names stable before changing branch protection settings.
- Release authorization is controlled by repository variables:
  - `RELEASE_AUTHORIZED_ACTORS`
  - `RELEASE_AUTHORIZED_TAGGER_EMAILS` (optional)
