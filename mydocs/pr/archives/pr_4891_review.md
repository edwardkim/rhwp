---
kind: pr-review
status: code-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4891 검토 - crash_minimizer 델타 축소 도구

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4891](https://github.com/edwardkim/rhwp/pull/4891) · @kevin9327 |
| 원 head | `8e7a1cd9db003af87d5575a4addfdb6f4a3454af` |
| 누적 적용 | source commit 2/17 · `4de9e5156` |
| 통합 기준선 | `upstream/devel@441254611` |

크래시 HWPX를 oracle 기반으로 축소하는 도구를 추가한다. 메인터너 보정 `51e9daa96`은 출력 경로가 입력과
같을 때 원본을 삭제·덮어쓸 수 있는 경로를 먼저 거부하고, 스트림 재설정 호환성을 분리했다. Python 계약
검증 13건과 #4918 Full CI·CodeQL을 통과했다. **수용 가능**이다.
