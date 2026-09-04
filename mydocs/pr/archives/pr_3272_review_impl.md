---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-25
---

# PR #3272 implementation 계획 — 최신 devel 위 메모 통합

## 적용 순서

1. 최신 `upstream/devel` `efc4b6c4c`에서 `review/lpaiu-cs-20260725`를 최신화했다.
2. performance 본체 `7c098e0d`를 `c0c566a4c`로 체리픽했다.
3. P1 BMP 표본 충돌 보정 `b873d66a`를 `640d41320`으로 이어 적용했다.
4. 원 branch의 devel merge commit `ab811440`은 최신 base가 이미 포함하므로 적용하지 않았다.

## PR 준비와 merge 후

1. review 기록과 `pr_3272_lpaiu-cs_issue2520_p001_review.png` asset을 docs commit으로 추가한다.
2. #2370의 수용 범위·maintainer 보정과 함께 하나의 `devel` 대상 integration PR을 만든다.
3. PR 본문에는 `Closes #2520`을 넣되, 실제 merge SHA와 issue close는 merge 뒤 확인한다.
4. 최신 integration head의 full CI·mergeable·작업지시자 승인을 다시 확인한 뒤 merge한다.
5. merge 뒤 #2520 close 상태를 확인하고, 원 #3272는 통합 PR로 대체된 사실을 comment한 뒤 close한다.

## rollback

- P1을 포함한 메모 전체를 되돌릴 때: `640d41320`, `c0c566a4c`를 역순으로 revert한다.
- asset·review 기록만 되돌릴 때는 docs commit만 revert한다. source/fixture/baseline 변경은 이 기록 commit에
  섞지 않는다.
