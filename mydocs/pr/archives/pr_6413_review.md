---
kind: pr-review
status: visual-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6413
issue: 6298
author: planet6897
---

# PR #6413 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 하지 않음 |
| 수용 대상 | 통합 PR #6481에 포함된 `12257188a7ed4a740e702fdc806af8434483af32` |
| 현재 상태 | 수용 보류: 통합 head visual sweep과 최신 CI가 아직 완료되지 않음 |
| 승인 뒤 처리 | #6481이 수용·병합된 뒤 #6413을 포함 수용 근거와 함께 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6413 |
| 원 head | `642354d174774074255cf701ff5a3753071c5b8b` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commit | `12257188a7ed4a740e702fdc806af8434483af32` |
| 통합 순서 | 1/8 |

## 검토

TAC 표의 leading 위치와 본문 우측 경계를 맞추는 #6298 회귀 보정이다. 원 PR CI는 수집 시점에 비성공 check 없이 완료된 상태였고, 최신 devel 위 cherry-pick에는 충돌이 없었다.

통합 후보에서 `twin_tac_tables_share_one_left_edge`, `tac_table_stays_inside_the_body_right_edge`가 통과했다. 공통 필수 검증인 native·WASM·workspace clippy, workspace build, rust test suite manifest check와 `cargo fmt --check`도 통과했다.

원 PR에 포함된 before/after PNG는 변경 의도를 확인하는 보조 자료로만 사용했다. 통합 head에서 독립 재생성한 시각 sweep은 아니므로 최종 렌더링 수용은 통합 PR의 Render Diff 성공을 조건으로 한다.

## 현재 결론

코드·계약 검증에는 차단 finding이 없지만, 사용자-visible table layout 변경이므로 visual sweep 없이 수용으로 확정하지 않는다. 현 원 PR은 직접 merge하지 않으며, 작업지시자 승인 뒤 #6481의 최신 CI와 visual sweep이 모두 확인된 경우에만 포함 수용 후 close한다.
