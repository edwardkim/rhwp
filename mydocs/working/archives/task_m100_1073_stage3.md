# Stage 3 보고서 — Task #1073: 측정·컷 레이어 구현

- 브랜치: `local/task1073`
- 수정: `src/renderer/layout/table_layout.rs`, `src/renderer/height_measurer.rs`

## 구현 (컷 레이어, 렌더 전)

### CellUnit + nested_row (table_layout.rs)
- `CellUnit` 에 `nested_row: Option<usize>` 추가(기존 3 사이트 = None).
- `cell_units`: **가시 텍스트 없는 문단(`p.text.trim().is_empty()`) + 단일 중첩 표 + 2행 이상**
  이면 per-중첩행 유닛으로 분해. height = row_h(+cs, +om_top/spacing_before 첫행,
  +om_bot/spacing_after 끝행) → **Σ = calc_nested_table_height (드리프트 0)**.
- 2단계+ 중첩 / 텍스트 동거 / rowspan 중첩행은 atom 폴백(범위 외).

### MeasuredCell.nested_split_row_count (height_measurer.rs)
- 신규 필드: 분해 가능한 중첩 표의 행 수(조건 동일). `is_row_splittable` 가
  `line_heights.len() > 1 || nested_split_row_count > 1` 로 중첩 표 행 분할 가부 인정.

### 자동 정합
- `row_cut_content_height` / `cell_line_ranges_from_cut` 는 cell_units 유닛 기반 → 추가 변경 없이
  per-중첩행 유닛 수용.

## 검증 (dump-pages — 핵심 발견)
- kps-ai pi=674: 외부 행 분할 `rows=0..2/2..3` → **중첩행 컷** `end_cut=[20] / start_cut=[20]`.
  page66 = 중첩행 0..20, page67 = 20..29. **LAYOUT_OVERFLOW 758px 해소.**
- 진단 경위: 중첩 표 para 는 `line_count=0` 이 아니라 `line_count=1`(placeholder) + `text=""`
  → 최초 `line_count==0` 조건 미스 → `text.trim().is_empty()` 로 정정.

## 전수 sweep (Stage 3 1차) — 회귀 0
baseline 3057 lines / 382815px → **3055 / 382054px**. 변화 파일 2개, **둘 다 개선**:
- kps-ai.hwp 10/849px → 9/90px (758 overflow 해소).
- hwpctl_ParameterSetID_Item_v1.2.hwp 6/53px → 5/51px (보너스 중첩표 케이스).
- 그 외 281 샘플 변화 0, 신규 overflow 0. **공유 컷 모델 변경 무회귀(페이지네이션 레벨).**
- lib **1324 passed**, clippy/fmt clean.

## 남은 Stage 4 갭 (렌더)
페이지네이션 컷은 정확하나 **렌더가 컷을 미반영** — page67(연속)이 `start_cut=[20]` 무시,
중첩행 0부터 재렌더(중복). 렌더의 `NestedTableSplit` 가 `available_h` 휴리스틱 기반이라
연속 페이지 start_row 미적용. → Stage 4: 컷(start_cut/end_cut) → NestedTableSplit 배선.

## 다음 (Stage 4)
부분 렌더 매핑 — 컷 유닛 인덱스 → 중첩행 범위 → NestedTableSplit{start_row,end_row,
offset_within_start}. kps-ai SVG 가 PDF p62~63 정합.
