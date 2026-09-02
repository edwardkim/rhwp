---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4890 검토 - clone 없이 GitHub 읽기·CI 점검 CLI

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4890](https://github.com/edwardkim/rhwp/pull/4890) · @kevin9327 |
| 원 head | `581dc24ffa3c243819924fc8d399c953ee7d3ee5` |
| 누적 적용 | source commit 1/17 · `ead273d8c` |
| 통합 기준선 | `upstream/devel@441254611` |

`gh_noclone` 읽기·SHA 조회·PR 상태 조회 도구를 추가한다. 메인터너 보정 `51e9daa96`에서 읽기 요청을 명시적
`GET`으로 고정하고 `--failed-only` 기본값 및 하위 명령 `--repo` 처리를 정정했다. 실제 REST 읽기와
Python 도구 계약 13건, 통합 PR #4918의 GitHub Full CI·CodeQL을 통과했다. **수용 가능**이다.
