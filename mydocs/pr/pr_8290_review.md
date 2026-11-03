---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-03
---

# PR #8290 검토 — 문단 간격 편집·되돌리기

## 최종 판정

**머지 보류 — 접수 완료, 현재 devel에서의 실행·시각 검증 진행 전.**
현재 단계는 원 head·기존 CI·코드와 실제 merge-tree의 접수 결과다. 원 head의 과거 통과를
최신 devel 통합 후보의 실행 완료로 보고하지 않는다. 아래 보정과 검증이 끝나면 판정을 갱신한다.

## 접수 정보

| 항목 | 고정값 |
| --- | --- |
| 원 PR / 작성자 | [#8290](https://github.com/edwardkim/rhwp/pull/8290) / semanticist21 |
| 원 repository·branch | `semanticist21/rhwp:fix/paragraph-spacing-vpos` |
| 원 head | `7e5b3f40d2f83ebba6c01b02be4327c564a28902` |
| 현재 기준 devel | `7d06d5c706f1d89f1662e739abf2b1b25d2a98e4` |
| Reviewer | edwardkim, 2026-11-03T19:19:38.880954+09:00 API 확인 |
| 원 head CI | commit status success; neutral 1, skipped 6, success 28 |
| 실제 merge-tree | clean |
| 로컬 branch | `review/integration-20261103-semanticist21` |
| 경로 | maintainer_general + intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge |

## 주장·현재 devel 대조와 보정 방침

문단 서식 변경 직후 그 문단과 뒤 문단의 저장 vpos를 갱신한다.

apply_para_format_native와 set_para_shape_id_native에 PR이 제안한 vpos 갱신이 있는지 대조했다. 원 head의 실패/성공 주장은 현재 base에서 재실행해야 한다.

본문 재조판·원래 end 전달·되돌리기와 #8299의 저장 줄 없는 문단 사이 공간 보존을 함께 검증한다. 새 빈 단·쪽 리셋·줄간격 대조를 유지한다.

이 절의 현재 관측은 코드·이력 대조이며 현재 base/head의 실제 재현 결과는 검증 단계에서 추가한다.

## 검증 범위

paragraph_spacing_moves_stored_lines; 앞/뒤 간격·이웃 문단·undo/redo·저장 재열기·줄 리셋

제품 경로:
- `src/document_core/commands/formatting.rs`

정식 focused 대상:
- `tests/cases/paragraph_spacing_moves_stored_lines.rs`

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
