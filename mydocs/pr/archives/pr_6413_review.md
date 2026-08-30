---
kind: pr-review
status: approved-via-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
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
| 현재 상태 | 승인: #6481 통합 후보 기준으로 코드·계약·claim-scoped 시각 증적에 차단 finding 없음 |
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

원 PR에 포함된 before/after PNG는 변경 의도를 확인하는 보조 자료로만 사용했다. 통합 code head에서 HWP MCP 2020 기준 PDF를 다시 만들고 12쪽을 직접 비교했다. `pixel_match=87.44435`, `visual_accuracy_proxy_percent=78.80165`였고, 자동 column line-band 후보 2건은 review PNG에서 표의 raster/text band 묶음 차이로 확인됐다. 표의 공통 좌측선과 본문 우측 경계에는 clipping 또는 overflow가 보이지 않았다. 대표 증적은 [p12 review PNG](../assets/pr_6481_issue6298_p012_review.png)이며, 명령·PDF SHA·임시 compare/overlay 경로는 [PR #6481 visual sweep 기록](pr_6481_planet6897_visual_sweep.md)에 보존했다.

## 현재 결론

**최종 판정: 승인.** TAC 표 경계라는 이번 주장에 대한 직접 시각 증적과 계약 검증에는 차단 finding이 없다. 수치는 전체 fidelity 합격이 아니라 claim-scoped 근거이며, 원 PR은 직접 merge하지 않고 #6481 통합 결과로만 수용한다. remote push, merge, #6413 close는 별도 지시가 있을 때만 수행한다.
