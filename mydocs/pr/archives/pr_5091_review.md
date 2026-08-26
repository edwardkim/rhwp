---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5091 검토 - vello_svg 0.10.0

## 접수와 범위

| 항목 | 기록 |
| --- | --- |
| PR | [#5091](https://github.com/edwardkim/rhwp/pull/5091) |
| 작성자 / source | `app/dependabot` / `dependabot/cargo/devel/vello_svg-0.10.0` |
| 원 source head | `85d8ba20a8564bce9a9a2220aa42aef5fbebf184` |
| 기준 / 규모 | `devel`, 2 files, +89 / -11 |
| 원 PR 상태 | 작성 시점 `MERGEABLE` / `CLEAN` |
| 통합 PR | [#5186](https://github.com/edwardkim/rhwp/pull/5186) |

`vello_svg` 0.7.1→0.10.0 갱신이며, 공유 `usvg` 그래프 정렬을 위해 `resvg-gpu`도 0.46으로 보정했다.

## 통합 적용과 검증

원 SHA를 `fb89fa0a0eadcd101815131f70ad1df8a08754bd`로 적용했다. 이어진 Vello/WGPU 호환성 이행은
`src/renderer/gpu.rs`에 한정했다.

- GPU feature `cargo check`, clippy, security trailer test(19 passed), full release-test nextest(6,522 passed, 38 skipped)를 통과했다.
- `samples/basic/english.hwp`를 GPU로 내보내 199×281 PNG를 생성했다. 이 호스트는 DX12 Microsoft Basic Render Driver를 사용했다.
- #5186 code candidate의 Canvas visual diff, Native Skia, CI·CodeQL이 성공했다.

## 판단

렌더 경로가 전이 의존성 API에 맞게 이행됐고 GPU smoke 및 CI 시각 검증을 통과했다. **통합 수용 권고.**
최종 문서 head에서도 동일 PR의 최신 Actions와 mergeability를 확인한다.
