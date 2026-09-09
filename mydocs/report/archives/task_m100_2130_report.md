# 최종 결과보고서 — Task M100 #2130: 리팩토링 산식 개정 (§5 v2.1)

- 이슈: #2130 (#1582 서브) / 브랜치: `local/task2130` / 2026-07-09

## 산출물 4종

1. **§5.1 v2.1 개정** (`plans/refactoring_plan_2026.md`): 지표 이원화 — 분포(최대/개수,
   현행)와 **총량(CC 총합·상위 20 합·>25 합, 신설)**. 라운드 성공 기준 = **총량 순감소**,
   통이동은 "준비 단계"로만 계상. 대상 선정 = "감소 잠재량 순 × 위험 낮은 순".
2. **감소 잠재량 스캐너** (`tools/reduction_potential.py`): 17라운드 실증 4유형
   (①중복 블록 R9형 ②지역 macro R16형 ③공통 guard 판정 체인 ④소스분기 밀집 —
   감사 Stage 1 교차). **보정 검증**: R9형 실후보가 상위, 포화 판정 함수(typeset_endnote
   등)는 하위 — 기준 충족.
3. **metrics.sh 보강**: 요약에 총량 3종 추가 + `--diff <스냅샷>` 함수별 CC 변화 표.
   검증 — phase2 기준값 정확 일치(11,701/1,726/4,384), r13→phase2 diff 가 R16(main.rs −51)
   과 유입 성장(shape.rs +12 등)을 정확히 표출.
4. **차기 후보 목록** (잠재량 순 상위, 아래) — 전부 종전 "최대 CC 순"에서는 안 보이던
   대상들. CC 는 낮아도 **총량 감소가 실제로 일어나는** 지점들이다.

## 차기 후보 상위 (잠재량 점수 = ①중복줄+②macro+③체인+④소스분기×4)

| 점수 | 함수 | 유형 | 비고 |
|---|---|---|---|
| 243 | `compute_char_positions` (text_measurement) | ①중복 | 최대 잠재 — 동형 블록 다수 |
| 134 | `build_char_properties_json_by_id` (formatting) | ①중복 | document_core 축 |
| 106 | `measure_table_impl` (height_measurer) | ①중복 | |
| 94 | `advance_row_block_cut` (table_layout) | ①중복 | 표 계열 — R10 scan 과 연접 |
| 94 | `export_markdown` (main.rs) | ①중복 | CLI — R16 처방 병행 가능 |
| 87+75 | `apply/format_shape_props_inner` (object_ops) | ①중복쌍 | 두 함수 간 동형 — 통합 후보 |
| 77 | `get_cursor_rect_by_path_with_hint` | ①중복 | #2021 산물 — 힌트/무힌트 경로 중복 |
| 36 | `paginate_pass` (rendering.rs, CC 105) | ④소스분기 | **잠재량·CC 둘 다 상위 — 최우선** |

## 게이트

소스 무접촉(문서+tools+스크립트). bash -n 구문 검증 + 기준값 일치 검증 완료.
