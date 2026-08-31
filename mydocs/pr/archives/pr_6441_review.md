---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6441
author: kevin9327
---

# PR #6441 review - 쪽 분할 셀 TAC 그림의 흐름 높이

## 검토 대상과 보정

- 원 PR head: `1612102b2ae2948bb2df3e7075be7048caf19d44`.
- 통합 적용 commit `98c42eeb` 뒤, 메인터너 보정 `694597b1`이 흐름 전진을 "쪽 분할 셀 안 TAC 그림"으로
  한정했다. 검증 base는 `upstream/devel@3afbb066fe93724ab44309163a2e04efb954bf18`이며, PR 직전
  `upstream/devel@cfa4ccacab63b470771720ebed33503cdd62adb6`로 충돌 없이 rebase했다.
- 2026-08-31 재조회에서 Open/non-draft이고 requested reviewer는 비어 있다. source head의 Build & Test,
  Lint, Native Skia, Archive A-D와 adapter/proptest는 성공했다.

## 검증

- 합성 fixture는 26px LINE_SEG에 312px 차트와 725px 별지 그림을 넣는다. 그림의 실제 paint height만큼
  셀 조각 회계와 `para_y`가 전진해 본문/아래 그림이 겹치지 않는지를 잠근다.
- 외부 HWP/HWPX 기준 PDF가 없는 합성 fixture이므로 visual sweep 대상은 아니다.
- 통합 후보에서 fmt, native/WASM clippy, workspace build, all-target clippy, manifest, unit tier check가
  통과했고, release-test 전체 nextest는 `8870 passed, 46 skipped` (450.949초, exit 0)였다.
- rebase는 충돌 없이 적용됐으며 추가 로컬 회귀는 수행하지 않았다. 최종 PR head의 CI 통과를 merge 조건으로 둔다.

## 판단과 후속 comment 계획

**수용(메인터너 보정 포함).** 넓은 TAC 그림 흐름 변경 대신 쪽 분할 셀의 실제 페인트 높이에만 적용했고,
312px/725px 회귀와 전체 회귀가 통과했다. 통합 PR merge 뒤 source PR에는 한정 predicate와 full nextest
결과를 기록한 뒤 integration 수용으로 close한다. 합성 fixture이므로 visual asset comment는 게시하지 않는다.
