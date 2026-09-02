---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4913 검토 - layer_tree 텍스트 원본 표 계약 테스트

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4913](https://github.com/edwardkim/rhwp/pull/4913) · @kevin9327 |
| 원 head | `7f911c377bb6bb16e71e2a1764205168092ded8f` |
| 누적 적용 | source commit 17/17 · `0be65d233` |
| 통합 기준선 | `upstream/devel@441254611` |

UTF-8/UTF-16 범위, annotation offset, 재귀, id 순서, 빈 tree와 source key 안정성을 고정하는 여섯 회귀
테스트를 추가한다. `paint::layer_tree::tests` 6건, 전체 nextest 6,383건, Native Skia와 Canvas visual
diff를 통과했다. **수용 가능**이다.
