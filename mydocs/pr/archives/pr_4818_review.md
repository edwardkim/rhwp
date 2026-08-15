---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4818 검토 - renderer layout i32 오버플로 하드닝

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4818](https://github.com/edwardkim/rhwp/pull/4818) · @kevin9327 |
| 원 head | `3e9661c5bd2dcbd17c87261dc5cb363f1eb729f3` |
| 누적 순서·적용 SHA | 1/27 · `163d35c94` → `c6a7ea47c` |
| 통합 기준선 | `upstream/devel@6631e7057` |
| 충돌·의존성 | 충돌 없음 · 독립 renderer 산술 보정 |

`vertical_pos + line_height`와 layout 산술의 i32 오버플로를 포화 연산으로 바꿔 손상 입력의 패닉을 막는다.
퍼징 재현 범위와 회귀 검사가 함께 적용됐다.

통합 후보의 전체 로컬 nextest 6,340건, Native Skia, OVR 기하 회귀 0건과 GitHub code CI
(`#4883`, `2f61f4167`)가 통과했다. **수용 가능**으로 판정한다.
