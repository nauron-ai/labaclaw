# Bản đồ CI Workflow

Repo hiện dùng bộ workflow CI/CD tối giản.

## Workflow đang hoạt động

- `.github/workflows/ci-run.yml`
  - Gate chất lượng cho PR/push.
  - Lint/test/build smoke cho Rust + kiểm tra chất lượng tài liệu.
  - Merge gate: `CI Required Gate`.

- `.github/workflows/release-build.yml`
  - Build binary production khi push `main`, tag `v*`, hoặc chạy thủ công.
  - Xuất artifact `zeroclaw-linux-amd64` (`zeroclaw` + `.sha256`).

- `.github/workflows/pub-release.yml`
  - Verify build (manual/schedule) và publish release khi tag stable `vX.Y.Z`.
  - Có guard kiểm tra trigger release và hợp đồng artifact.

## Lưu ý vận hành

- Docker publish đã bị tắt có chủ đích.
- Quyền publish release được kiểm soát bởi `RELEASE_AUTHORIZED_ACTORS`.
