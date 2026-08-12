---
kind: pr-review
status: local-accept-recommendation
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4555 리뷰 - 끝 앵커 인라인 표 wrap

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4555](https://github.com/edwardkim/rhwp/pull/4555) · @planet6897 |
| base / 원 head | `devel` / `e892b31f697b915356b4b349fb58f6f1ed7f841f` |
| 규모 | 2 files, `+154/-1`, 2 commits |
| 작성 시점 상태 | OPEN, `MERGEABLE`, `CLEAN` (merge 직전 재확인 필요) |

## 검토와 판정

문단 끝(`position == text_len`)의 treat-as-char 표가 남은 폭을 넘을 때 다음 줄로 wrap하도록
중간 앵커 조건을 확장한다. 선두 앵커와 폭 안쪽 표의 기존 동작은 focused 4건으로 함께 고정했다.

`issue_4370_bottom_overflow_reflow` 4건은 모든 페이지의 frame 안 배치와 다음 쪽 내용 보존을
확인했다. Native Skia 3종, release-test 전체, wasm-pack 산출 package 로드까지 누적 branch에서
성공했다.

**최종 권고: 최신 통합 head의 CI와 작업지시자 승인 후 수용.**
