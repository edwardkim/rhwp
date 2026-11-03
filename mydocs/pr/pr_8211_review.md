---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8211 검토 — 표 삽입·원본 본문 흐름

## 최종 판정

**머지 보류 — 접수 완료, 현재 devel에서의 실행·시각 검증 진행 전.**
현재 단계는 원 head·기존 CI·코드와 실제 merge-tree의 접수 결과다. 원 head의 과거 통과를
최신 devel 통합 후보의 실행 완료로 보고하지 않는다. 아래 보정과 검증이 끝나면 판정을 갱신한다.

## 접수 정보

| 항목 | 고정값 |
| --- | --- |
| 원 PR / 작성자 | [#8211](https://github.com/edwardkim/rhwp/pull/8211) / semanticist21 |
| 원 repository·branch | `semanticist21/rhwp:codex/table-measurement-ownership` |
| 원 head | `82b959af244dfdbfeaffb4186910c371f2afdc29` |
| 현재 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| Reviewer | edwardkim, 2026-11-03T19:19:22.646529+09:00 API 확인 |
| 원 head CI | commit status success; skipped 4, success 31 |
| 실제 merge-tree | 충돌 5파일 |
| 로컬 branch | `review/integration-20261103-semanticist21` |
| 경로 | maintainer_general + intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge |

## 주장·현재 devel 대조와 보정 방침

표 삽입 뒤 측정 주소 이동과 공백 호스트·저장 행 프레임 보존을 추가한다.

현재 삽입 경로에 PR의 측정 주소 이동을 더해야 하는지 실제 53개 호출로 대조한다. 저장 행 프레임은 현재의 행별 dirty·원본 소유 검사를 보존한다.

측정 주소 이동을 공통 삽입 경로에 연결하고 현재 TAC 제어 전용 꼬리·숨긴 첫 줄·쪽 원점·분할 host handoff를 보존한다. 마지막 저장 표 줄의 간격은 같은 줄 소유 결과에서 한 번만 계상한다.

이 절의 현재 관측은 코드·이력 대조이며 현재 base/head의 실제 재현 결과는 검증 단계에서 추가한다.

충돌 파일:
- `src/renderer/layout/table_layout.rs`
- `src/renderer/typeset.rs`
- `src/renderer/typeset/controls/order.rs`
- `src/renderer/typeset/section/vpos.rs`
- `src/renderer/typeset/table/block/entry.rs`

## 검증 범위

table_insert_measurement_ownership, table_insert_body_flow; 공개 53개 호출의 live·HWPX 재열기; 원본 뒤 본문·중첩 표·CellBreak·Center·None/Square 대조

제품 경로:
- `src/document_core/commands/object_ops/table.rs`
- `src/renderer/float_placement.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_partial.rs`
- `src/renderer/typeset.rs`
- `src/renderer/typeset/controls/order.rs`
- `src/renderer/typeset/controls/stored_tac.rs`
- `src/renderer/typeset/paragraph.rs`
- `src/renderer/typeset/paragraph/content.rs`
- `src/renderer/typeset/section/vpos.rs`
- `src/renderer/typeset/table/block/entry.rs`

정식 focused 대상:
- `tests/cases/table_insert_body_flow.rs`
- `tests/cases/table_insert_measurement_ownership.rs`

원본·독립 Print·편집 상태의 대응을 확인하고 현재 base/후보의 Native/fresh WASM 전쪽 TSV와 확대 판독을
확보한다. 입력이 합성 계약인 범위와 실물 한컴 출력은 구분한다. 현재 단계에 새 측정 수치를 적지 않는다.

## 조판 원칙 준수 검토

| 항목 | 접수 판정과 후속 근거 |
| --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 기존 이슈·해결 PR의 선행 분석 | 미검증 — 위 현재 devel 관측과 #8093/#8187/#8517의 선행 연결을 시작했으며 적용 경계의 실행을 남긴다. |
| 구현 근거와 일반성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 측정·배치 일관성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 리베이스 반영 PR의 기능 보존 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 최종 출력 소비 계약 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 분할·이어받기 계약 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 줄 소속과 점유 높이 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 사례와 증거의 독립성 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 기준값 변경 | 비해당 — 접수 단계에서 코드·기대값·baseline·golden을 변경하지 않았다. 이후 검사 변경은 독립 출력 gate 뒤에 수행한다. |
| 회귀 추가·변경의 선행 조건 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 90% 통과 후 확대 판독 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |
| 주장과 검증 범위 | 미검증 — [누적 검토 계획](semanticist21_20261103_review_plan.md)의 해당 실제 경로·입력·독립 근거를 검증한다. |

## Merge 후 contributor PR comment 계획

현재는 maintainer 시각 검증 미실행이다. 검증 완료 뒤 이 원 PR의 실제 결론·source/merge SHA·CI URL,
[Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)과 실제 측정한 전쪽 최저값·직접 판독을
연결한다. 수용 시 원 기여와 메인터너 보정 이유를 존댓말로 구분하고 정식 대표 PNG가 devel에 반영된 뒤
승인된 원격 범위에서 게시·API 재확인한다. 아직 없는 수치나 이미지 URL은 만들지 않는다.

[누적 검토 계획](semanticist21_20261103_review_plan.md) · [원 head·CI·충돌 증거](assets/semanticist21-20261103/intake.json).


## 첫 통합의 보정 — 실행 검증 전

- 원 head를 merge parent로 보존한다. 삽입 경로의 측정 주소 이동을 적용했다.
- 원본 셀 인증과 행별 reflow 소유를 함께 유지했다. 기존 제어 전용 TAC 꼬리와 마지막 표 줄 인증은 서로의 줄 소유 조건을 보존한다.
- 첫 문단 원점의 생산을 `initialize_first_paragraph_origin`에 모아 일반·분할 진입과 vpos 스냅이 같은 hidden-line/Square/합성 사다리 계약을 소비한다. 반환된 원점 하한은 기존 vpos 소비에 전달한다.
- 공백/표 원시 stream의 완전성 검사는 Storage의 슬롯 조회·배열·정렬·push 경로에 연결했다.
- None 표/혼합 host는 최신 `unrestricted_float`와 split host handoff를 보존했다. 실제 53개 호출의 잔여 차이를 원본 Print·반례와 대조하기 전에는 완전 해결로 판정하지 않는다.
