---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# PR #2370 implementation 계획 — 누적 체리픽 통합

## 포함 순서와 경계

1. `upstream/devel` `efc4b6c4c`에서 가시성 branch `review/lpaiu-cs-20260725`를 최신화했다.
2. #2370의 기능 commit `87f196b2`만 `98f7cd453`으로 적용했다. source branch의 devel merge commit은
   최신 base가 이미 포함하므로 중복 적용하지 않았다.
3. #3272와 충돌 없이 누적한 뒤, #2370 메인터너 지시를 `fcfc966fa`(코드·source guard)과
   `40f2f4a3d`(browser E2E)로 분리했다.

## PR 준비 단계

1. 이 review 기록, #3272 개별 기록·visual asset을 별도 docs commit으로 추가한다.
2. 변경은 `devel` 대상 integration PR 하나로 올린다. 원 #2370 branch에는 push하지 않는다.
3. PR 본문은 contributor 원 변경, #3272 변경, maintainer 보정을 구분하고 #2370 tracker를 close하지
   않는다고 명시한다.
4. 최신 integration head의 full CI, mergeable 상태, 작업지시자 승인 뒤에만 merge한다.

## rollback

- Escape/방어 주석 보정만 되돌릴 때: `fcfc966fa`를 revert한다. E2E commit `40f2f4a3d`는 함께
  되돌리거나 독립 회귀 기록으로 유지할지 결정한다.
- #2370 중복 emit 정리만 되돌릴 때: `98f7cd453`만 revert한다.
- 아직 원격 push·PR·comment는 없으므로 현재 단계의 rollback은 이 local integration branch에서만 한다.
