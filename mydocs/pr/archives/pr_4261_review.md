---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4261 검토 - 거대 분할 셀 Enter 지연 제거

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4261](https://github.com/edwardkim/rhwp/pull/4261) / @humdrum00001010 |
| contributor 원 head | `59aef443c37e804ae2cf18e5c5d4e80cedc566bd` |
| base / 규모 | `devel`, 31개 파일, +2,132/-41 |
| 관련 이슈 | [#4146](https://github.com/edwardkim/rhwp/issues/4146) |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |

거대 분할 표 셀에서 Enter 후 남은 deferred pagination을 계산 없이 취소하고, split 완료 소유를 command
effects로 옮긴다. probe와 실제 browser E2E를 함께 추가한다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- Studio focused 27건, 전체 `npm test` 813건, production build가 통과했다.
- 실제 Chromium E2E에서 HWP/HWPX 모두 115쪽, Enter flush 0, split 1, ArrowDown barrier flush 1을
  확인했다.
- 전체 `release-test --tests`와 WASM build가 통과했다.

**통합 수용 권고.** #4258 → #4259 → #4260 이후에만 적용해 stale line segment와 저장 캐럿 계약을
먼저 확정한다.
