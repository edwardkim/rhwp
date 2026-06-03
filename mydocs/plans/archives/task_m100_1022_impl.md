# 구현계획서 — HeightMeasurer ↔ cell_units 측정 정합

- 타스크: GitHub #1022 (M100, v1.0.0)
- 브랜치: `local/task1022`
- 작성일: 2026-05-20
- 선행: 수행계획서 `task_m100_1022.md`

## 1. 측정 시스템 — 현 위치 정리

세 경로가 셀 콘텐츠 높이를 산출한다:

| 경로 | 위치 | 호출처 |
|------|------|--------|
| A. `HeightMeasurer::measure_table_impl` → `MeasuredCell` → `MeasuredTable.row_heights` | `src/renderer/height_measurer.rs:494` | 페이지네이션 사전 측정, 렌더러 `resolve_row_heights` |
| B. `LayoutEngine::calc_cell_paragraphs_content_height` | `table_layout.rs:1028` | `resolve_row_heights` 폴백(MeasuredTable 없을 때) |
| C. `LayoutEngine::cell_units` (+ `advance_row_cut`) | `table_layout.rs:3405+` | task993 컷 walk, `row_cut_content_height` |

베이스라인(task992): paginator·renderer 모두 A 권위(`mt.range_height` 누적·`resolve_row_heights` 반환). 일관 → `LAYOUT_OVERFLOW` 0.

task993: paginator C(`advance_row_cut` 누적), renderer 분할 행만 C(`cell_line_ranges_from_cut`)·비분할 행 A. A vs C 차이가 누적 → 50건 회귀.

## 2. 단계 구성

### Stage 1 — 측정 알고리즘 비교 조사

- A·C 의 줄 단위 높이 계산식 추출(`MeasuredCell.line_heights[i]` 산식 vs `cell_units[u].height` 산식).
- 차이 항목 분류:
  - `corrected_line_height` 적용 여부.
  - `line_spacing` 가산 위치(줄 끝 vs 줄 시작 vs 셀 마지막 줄 제외 규칙).
  - `spacing_before` / `spacing_after` 적용 위치(문단 첫·마지막 줄, 셀 첫·마지막 문단).
  - `cell.height` 필러(C 만 적용).
  - vpos 동기화(`compute_cell_line_ranges` 의 `[Task #700]` 가드) 영향.
  - 중첩 표·빈 문단 atom 측정.
- 단계 보고서: A·C 산식 표 + 차이 원인 분류.

### Stage 2 — 정합 설계

세 후보안 비교:

| 후보 | 방향 | 장단 |
|------|------|------|
| (a) C → A | `cell_units` 를 `MeasuredCell.line_heights` 기반으로 재구성 | 베이스라인 LAYOUT_OVERFLOW 0 복원. cell_units 가 HeightMeasurer 의존 — 순환 가능성 점검 필요. |
| (b) A → C | `MeasuredCell` 를 `cell_units` 기반으로 재구성 | 단일 권위가 cell_units. HeightMeasurer 범위 광범위 — 회귀 위험 큼. |
| (c) 공통 보조 함수 | A·C 모두가 호출하는 셀 콘텐츠 측정 헬퍼 추출 | 가장 깔끔하나 두 모듈 모두 변경 — 작업량 큼. |

기본안 (a) — 변경 범위 최소·베이스라인 동치 복원 직접. Stage 1 결과로 확정.

### Stage 3 — 구현

기본안 (a) 기준:
- `cell_units` 를 `MeasuredCell.line_heights` / `total_content_height` 직접 참조하도록 재구성. 필러 규칙도 MeasuredCell 의 cell.height 처리에 맞춰 통일.
- `advance_row_cut` 의 hard break(vpos 리셋) 검출은 보존 — `cell_units` 의 보조 메타에서 line_segs 기반으로 계산.
- `row_cut_content_height` / `cell_line_ranges_from_cut` 도 새 측정으로 재배선.

### Stage 4 — 검증 (Stage 3 부분 정합 결과)

- ✅ 비공개 184페이지 `LAYOUT_OVERFLOW` 42→**38건**.
- ✅ form-002 한컴 2022 PDF 정합 유지(부동소수 갱신).
- ✅ `cargo test --release` 1302 passed, `cargo clippy --release` 무경고.
- ❌ 페이지 22 18.3px 잔존 — 원인: **VPOS_CORR ↔ paginator 미정합**
  (`layout.rs:2455` 의 HWP LINE_SEG.vpos 기반 위치 보정과 paginator 의
  `MeasuredParagraph.total_height` 누적이 다름).

### Stage 5 — VPOS_CORR ↔ paginator 정합 (사용자 결정에 의해 본 타스크에 포함)

#### Stage 5-1: VPOS_CORR 감사

- `layout.rs:2455` 의 VPOS_CORR 블록 (`vpos_lazy_base`, `is_page_path`,
  `prev_pi`, `seg.vertical_pos`, MAX_BACKWARD_PX, stale_table_host 등)
  전체 케이스 정리.
- 어떤 paragraph/Table 항목에서 발동되는지, 어떤 항목에서 비활성인지.
- 발동 조건 매트릭스 작성.

#### Stage 5-2: 정합 방향 결정

후보:
- (방향 1) **paginator 가 vpos 추적**: `MeasuredParagraph` 에 vpos 절대값
  보관 후 paginator 의 `current_height` 누적 시 vpos 기반 차분으로 정정.
- (방향 2) **VPOS_CORR 비활성화**: 렌더러가 LINE_SEG.vpos 무시,
  `MeasuredParagraph.total_height` 만으로 advance. 회귀 검토 필수.

#### Stage 5-3: 구현

선택한 방향으로 구현. paginator·renderer 단일 위치 모델.

#### Stage 5-4: 광범위 회귀 검증

- 공개 분할 표 골든 SVG 전체 회귀 점검.
- form-002 PDF 정합 유지 확인.
- task993 컷 모델 invariants 보존(form-002 vpos 분할 정합, 무한 루프 차단).
- 비공개 184페이지 LAYOUT_OVERFLOW 0건 확인.
- WASM 영향 점검.

### Stage 6 — 최종 결과보고서

`report/task_m100_1022_report.md` 작성. WASM 재빌드 명시.

## 3. 영향 범위 / 리스크

- 변경 파일(예상): `src/renderer/layout/table_layout.rs`(cell_units / row_cut_content_height / cell_line_ranges_from_cut 재구성), 최소 가능 시 `src/renderer/height_measurer.rs` 일부.
- 후보안 (a) 가 cell_units 만 변경하면 페이지네이터·렌더러 정합이 자동 회복.
- vpos 리셋 검출은 `MeasuredCell` 가 정보를 보존하는지 Stage 1 에서 확인 — 미보존이면 `cell_units` 가 line_segs 직접 참조 유지.
- 측정 통일 후 일부 골든 SVG 가 미세 이동 가능 — Stage 4 에서 PDF 대조 판정.

## 4. 비공개 문서

비공개 184페이지 샘플은 커밋·문서 기재 금지. 회귀 검증은 스캔 결과만 보고서에 기록.
