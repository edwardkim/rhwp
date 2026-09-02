---
kind: pr-review
status: code-ci-running
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4917 검토 — HWPX 열거 밖 필드 종류의 정체성 보존

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4917](https://github.com/edwardkim/rhwp/pull/4917), @planet6897 |
| 원 head | `0cccebc593ef6901c52fa870d561deed51978548` |
| 통합 적용 | `59e599bdc` |
| 기준 | `upstream/devel@ae5f2a345` |

열거형에 없는 HWPX field type을 `CROSSREF`로 강제하지 않고 원본 raw type을 보존한다. 알려진 값은 기존
열거형과 호환되고, serializer는 raw 값·명령·control id 순으로 안전한 fallback을 사용한다. 알려지지 않은
필드 종류와 기존 CROSSREF의 round-trip 회귀 2건, 통합 전체 게이트가 통과했다.

모델·직렬화 계약 변경으로 독립 시각 fixture는 요구되지 않는다. 통합 PR
[#4936](https://github.com/edwardkim/rhwp/pull/4936)의 최초 코드 후보 CI는 녹색이었다. 최신 devel 동기화 뒤
docs head의 필수 CI와 head 동일성을 다시 확인하면 **수용 가능**이다.
