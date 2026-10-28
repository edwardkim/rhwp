---
kind: review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6316 검토 - text overlap ratchet gate

- PR: [#6316](https://github.com/edwardkim/rhwp/pull/6316)
- 이슈: [#6315](https://github.com/edwardkim/rhwp/issues/6315)
- 작성자: `@planet6897`
- 원 source head: `efaa1ae32852255c55773a245f1edcd0a935e044`
- 누적 검토 적용: `ffb05c592` (`git cherry-pick -x`)

## 변경 검토

문서별 text overlap baseline TSV와 `tests/cases/text_overlap_baseline.rs`를 추가해, 기준보다 overlap
수가 증가하는 회귀를 막는다. 문서 전체의 `scan_document` 결과를 사용하므로 스캔 실패 시 기존 nonzero
baseline 행이 누락되어 통과로 위장되지 않는다.

## 검증 상태

- 원 source head의 GitHub required CI가 성공했다. Rust CodeQL은 12분 45초, test archive B/C/D와
  native·lint·adapter·proptest·render-diff를 포함한 완료 check가 모두 green임을 확인했다.
- 누적 체리픽은 충돌 없이 적용됐다.
- 로컬 회귀와 PDF·visual sweep은 이번 누적 정적 검토에서 추가 실행하지 않았다.

## 최종 판정 - 수용 가능

`#6316`은 수용 가능하다. 다른 누적 후보의 P1/P2 보정이 끝나고 통합 head CI가 성공하면 함께
merge 대상으로 확정한다.
