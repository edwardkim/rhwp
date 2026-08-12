---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4538 리뷰 - 검증 harness

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4538](https://github.com/edwardkim/rhwp/pull/4538) · @kevin9327 |
| base / 최신 head | `devel` / `f7070666cd4fb1f4170ecc0b12b326bec2ab1f9c` |
| 규모 | 2,807 추가 / 27 삭제, 22 파일, 6 commit |
| 작성 시점 CI | GitHub required `Build & Test` 통과. merge 전 최신 상태 재확인 필요. |

## 범위와 메인터너 보정

`harness init/wrap`과 읽기 전용 `harness-status`를 분리해 검증 체인을 단일 계약으로 만든다.
후속 source의 `harness-status` 분리·MCP 주석·provenance 레시피 수정은 누적 branch에 이미
patch-equivalent하게 존재함을 확인했다. 따라서 중복 merge commit은 적용하지 않았다.

**권고: 최신 head와 required check를 merge 직전에 재확인하고 통합 수용.**
