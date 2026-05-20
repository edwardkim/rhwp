# 구현계획서 — #1025: 페이지보다 큰 단일 표 셀 내부 분할

- 이슈: #1025 / 브랜치 `local/task1025`
- 수행계획서: `task_m100_1025.md` (승인 완료)

## 핵심 원인 (코드 확정 — Stage 1 line 갱신: base edf62865)

`advance_row_cut`(table_layout.rs)는 **`row_span==1` 셀만** 순회한다(908/911). 따라서 rowspan(rs>1) 셀이 걸친 행은 컷 모델로 측정 불가.

typeset.rs:2539~2560이 이를 우회한다:
```rust
let rowspan_touched: Vec<bool> = ... c.row_span > 1 && c.row <= r < c.row + c.row_span ...
let cut_row_h[r] = if rowspan_touched[r] { mt.row_heights[r] }   // 원자적·분할불가
                   else { row_cut_content_height(...) };          // 줄 단위 분할 가능
```

**pi=324**: cell[6] r=3 **rs=2** 가 rows 3·4를 걸침 → 두 행 모두 `rowspan_touched`. row 4 의 거대 세부내용 셀(cell[10] rs=1, 1024px)이 들어있으나, row 4 가 rowspan_touched 라서 `mt.row_heights[4]`(~1024px)를 **원자 단위**로 취급 → 페이지보다 커도 쪼개지 못하고 통째로 배치 → 143px 초과·잘림.

→ **rowspan 셀이 걸친 행도 줄 단위로 분할**할 수 있게 컷 모델을 확장하는 것이 본질.

## 단계 (5단계)

### Stage 1 — 진단 + 한컴 PDF 정답 확인
- pi=321/323/324 의 `rowspan_touched` / `cut_row_h` / row_block 경계를 `dump-pages`·임시 로깅으로 측정, 위 원인 확정.
- **한컴 2022 PDF**(`pdf/hwpx/...` 또는 본 문서 PDF)에서 해당 표가 실제로 어떻게 분할되는지 판정:
  - 거대 세부내용 셀을 줄 단위로 페이지 경계에서 쪼개는가?
  - rs=2 "상세설명" 라벨 셀은 분할 경계에서 어떻게 표시되는가(반복? 잘림? 첫 페이지만?)?
- 결과로 분할 규칙(정답)을 문서화. → `working/task_m100_1025_stage1.md`

### Stage 2 — 컷 모델 rowspan 확장 설계
- `advance_row_cut` / `row_cut_content_height` / `cell_line_ranges_from_cut` 가 rowspan 셀을 포함하도록 설계:
  - rowspan 셀의 units 를 걸친 행들에 걸쳐 "잔여 진행"으로 추적(셀별 누적 컷 인덱스).
  - row-cut 1회는 해당 행의 rs==1 셀 + 그 행에서 진행 중인 rowspan 셀의 부분을 함께 소비.
- `RowCut`/`RowCutResult` 스키마 변경 여부 결정(셀 인덱스 정렬에 rowspan 셀 포함 시 정합 규칙).
- `rowspan_touched` 원자 처리 → "분할 가능" 전환 조건(셀 1개라도 page-larger 일 때) 정의.
- 한컴 동작과의 정합 확인. → `working/task_m100_1025_stage2.md`

### Stage 3 — 페이지네이터(typeset) 구현
- typeset.rs 컷 walk(2535~2800)에서 rowspan_touched 행을 Stage 2 설계대로 줄 단위 분할.
- `cut_row_h` / `header_overhead` / split_end_cut 산정에 rowspan 셀 반영.
- 단일 권위(`advance_row_cut`) 유지 — 페이지네이터·렌더러 공유. → `working/..._stage3.md` + 소스 커밋

### Stage 4 — 렌더러(layout_partial_table) 정합
- table_partial.rs 가 rowspan 셀의 분할 컷을 동일하게 그림(셀 내부 줄 범위 = `cell_line_ranges_from_cut`).
- rs=2 라벨 셀의 분할 경계 표시를 Stage 1 PDF 정답대로. → `working/..._stage4.md` + 소스 커밋

### Stage 5 — 검증 + 골든 + 회귀
- 비공개 184p `LAYOUT_OVERFLOW` 재측정: page-larger 셀 부류(pi=321/323/324 등) 제거 확인.
- `cargo test` 전체 + `svg_snapshot` 무회귀. 신규/이동 골든은 한컴 2022 PDF 판정.
- 광범위 회귀 sweep(다른 분할 표 문서). → `working/..._stage5.md` + 소스 커밋
- 최종 보고서 `report/task_m100_1025_report.md`.

## 리스크·완화
- 스키마/단일권위 함수 변경 → #993/#1022 분할 전반 영향. 각 단계 골든·테스트로 회귀 차단.
- rowspan 부분 분할의 한컴 동작이 불명확하면 Stage 1 PDF 판정 결과를 정답으로 고정 후 진행.
- 내부표 보유 셀(pi=323 cell[10] p[3]) 분할은 Stage 2에서 별도 케이스로 다룸(필요 시 내부표는 원자 유지).

## 비범위
- 표 외 page-larger 문단/그림. WASM 재빌드(릴리즈 시점).
