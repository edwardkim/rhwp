---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4912 검토 - 도형 전용 문단 판정 정렬

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4912](https://github.com/edwardkim/rhwp/pull/4912) · @kevin9327 |
| 원 head | `482a366c8b340461cf3b17c68cf57e56b602c759` |
| 누적 적용 | source commit 16/17 · `814a92ff8` |
| 통합 기준선 | `upstream/devel@441254611` |
| 관련 이슈 | [#4626](https://github.com/edwardkim/rhwp/issues/4626) 참조만 함 |

typeset과 height cursor의 도형 전용 문단 판정을 일치시켜 U+FFFC placeholder만 있는 문단을 올바르게
처리한다. focused `renderer::height_cursor::tests` 55건, 실제 placeholder PDF 회귀 2건, Native Skia,
전체 nextest 6,383건과 Canvas visual diff를 통과했다. #4626은 후속 #4333의 선행 추적 항목이므로 닫지
않는다. **수용 가능**이다.
