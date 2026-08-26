---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4550 리뷰 - lineage bundle

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4550](https://github.com/edwardkim/rhwp/pull/4550) · @kevin9327 |
| base / 최신 head | `devel` / `b466cb796d10407df77e6b0a61ed56d4681c2f68` |
| 규모 | 12,564 추가 / 100 삭제, 88 파일, 30 commit |
| 작성 시점 CI | GitHub required `Build & Test` 통과. merge 전 최신 상태 재확인 필요. |

## 범위와 메인터너 보정

`.lineage-bundle` export/verify와 오프라인 다단 판정을 추가한다. 후속 검증 사다리 명령까지 포함해 Node
envelope 생성 타입은 52개 봉투 헤더로 재생성했다. 생성 코덱스에는 `bundle`만 최상위로 기록하고 export/verify는 실제
도움말 경로로 보존해 synthetic command를 만들지 않도록 보정했다.

**권고: 최신 head와 required check를 merge 직전에 재확인하고 통합 수용.**
