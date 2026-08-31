---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6427
author: kevin9327
---

# PR #6427 review - 강제 줄나눔 뒤 인라인 개체의 저장 줄 경계

## 검토 대상과 보정

- 원 PR head: `77d59943b450c4d465d925c9a29e781b19076204`, 통합 적용 최종 commit `489b44542`.
- 기준 base: `upstream/devel@3afbb066fe93724ab44309163a2e04efb954bf18`.
- source PR은 base 변경으로 dirty이고 최신 head에는 실행 가능한 CI가 붙지 않았다. 2026-08-31 재조회에서
  Open/non-draft, `postmelee` requested reviewer 없음이 확인됐다.
- `composer.rs` 충돌은 현재 devel의 인라인 control 탐색 가시성을 유지하면서, `\\n` 직후 실제 TAC inline
  object가 다음 `LINE_SEG`를 시작할 때만 저장 경계를 보존하도록 합성했다. 일반 강제 줄나눔을 넓게 바꾸지
  않았고 빈 post-line을 만들지 않는 source PR의 최종 제한을 유지했다.

## 검증

- `tests/cases/issue_6300_hardbreak_inline_object.rs`가 TAC 표와 TAC 도형 모두에서 저장 `text_start=119`를
  유지하고 83 중복을 금지한다. 외부 HWP/HWPX 기준 PDF가 없는 모델 합성 fixture이므로 visual sweep 대상은 아니다.
- 통합 후보에서 fmt, native/WASM clippy, workspace build, all-target clippy, manifest 및 unit tier check가
  통과했고, release-test 전체 nextest는 `8870 passed, 46 skipped` (450.949초, exit 0)였다.

## 판단과 후속 comment 계획

**수용(메인터너 충돌 보정 포함).** 보호 조건이 실제 TAC 개체와 저장 line segment 경계로 좁고, 두 인라인
개체 회귀와 전체 회귀가 통과했다. 통합 PR merge 뒤 source PR에는 적용 source head, 충돌 보정의 범위,
full nextest 결과를 남긴 뒤 integration 수용으로 close한다. 합성 fixture이므로 visual asset comment는 없다.
