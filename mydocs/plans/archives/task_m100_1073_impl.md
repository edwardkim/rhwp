# 구현계획서 — Task #1073: 중첩 표 행 단위 다중 페이지 분할 (전면 구현 A)

- 이슈: edwardkim/rhwp#1073
- 브랜치: `local/task1073` (stream/devel `be2a71c4` 기준)
- 수행계획서: `task_m100_1073.md` (승인) / Stage 1 조사: `working/task_m100_1073_stage1.md` (승인)

## 목표
셀 안의 페이지보다 큰 중첩 표를 **중첩 표 행 경계에서 페이지에 걸쳐 분할**한다
(한컴 2022 정합 — `pdf/kps-ai-2022.pdf` p62~63). kps-ai pi=674 758px overflow 해소.

## 변경 대상 4 레이어 (Stage 1 확정)
1. **측정** `MeasuredCell` (`height_measurer.rs`): 중첩 표 셀의 분할 단위(중첩행 높이) 노출.
2. **분할 가부** `is_row_splittable`(`height_measurer.rs:1490`): 중첩 표 셀을 splittable 로 인정.
3. **컷 모델** `cell_units`(`table_layout.rs:3519`): 중첩 표를 per-중첩행 유닛으로 분해 +
   `row_cut_content_height`(`table_layout.rs:3862`) 높이 정합.
4. **부분 렌더** `table_partial.rs`: typeset 컷 → `NestedTableSplit`(중첩행 범위) 매핑을 페이지
   chunk 별로 배선(현재 단일 셀 available_h 기반 → 컷 구동으로 전환).

## CellUnit 의미 확장 (설계 핵심)
현재 `vis_start/vis_end` = 문단 줄 범위. 중첩 표 유닛은 **nested-row 범위**가 필요 →
`CellUnit` 에 유닛 종류(텍스트줄 / 중첩행) 식별 + nested row index 보강. 렌더가 컷의
nested-row 범위를 `NestedTableSplit{start_row,end_row,offset_within_start,...}` 로 변환.

## 단계 (Stage 2~5)

### Stage 2 — 설계 + 페이퍼 검증 (소스 무변경)
- CellUnit 확장 스키마 + 4 레이어 인터페이스 확정(측정→가부→컷→렌더 데이터 흐름).
- 한컴 정합 기준(중첩행 경계 분할, 부분 행 미발생) + 비회귀 케이스(중첩 표가 페이지에 들어가는
  기존 문서 — nested-table-border/688 등) 모순 점검 표.
- 산출물: `working/task_m100_1073_stage2.md`.

### Stage 3 — 측정·컷 레이어 (렌더 전, 페이지네이션 검증)
- MeasuredCell + is_row_splittable + cell_units + row_cut_content_height 가 중첩행 단위 인식.
- 검증: `dump-pages` 로 kps-ai pi=674 가 중첩행 경계로 **올바른 청크**(p62/p63 대응)로 분할되는지
  확인(렌더 매핑 전이라 시각은 미완 가능). 전수 sweep 로 페이지수 변화·회귀 1차 측정.
- 산출물: 소스 + `working/task_m100_1073_stage3.md`.

### Stage 4 — 부분 렌더 매핑
- 컷(nested-row 범위) → `NestedTableSplit` 배선. 각 페이지 chunk 가 정확한 중첩행 범위 렌더
  (외곽 박스/테두리·셀 spacing 정합 포함).
- 검증: kps-ai SVG 가 PDF p62~63 와 시각 정합(작업지시자 판정 보조: PDF 권위자료).
- 산출물: 소스 + `working/task_m100_1073_stage4.md`.

### Stage 5 — 회귀 검증 + 가드
- kps-ai overflow 758px 해소. 전수 sweep(samples) LAYOUT_OVERFLOW 합 **회귀 0**.
- 골든 SVG 8, `cargo test --release`(lib+통합), clippy/fmt. 회귀 가드 `tests/issue_1073_*.rs`
  (kps-ai 영향 페이지 text/표 max_y ≤ 페이지 높이 + 중첩행 분할 청크 검증).
- 산출물: `working/task_m100_1073_stage5.md` → `report/task_m100_1073_report.md`.

## 완료 기준
중첩 표가 페이지 경계에서 중첩행 단위 분할(PDF 정합) + kps-ai overflow 해소 + 비회귀 0 + 골든 0
+ 회귀 가드.

## 리스크
- `cell_units`/`advance_row_cut`/`row_cut_content_height`/`is_row_splittable` 는 **모든 표 분할
  공유** → 광범위 회귀. Stage 3 에서 dump-pages + 전수 sweep 으로 단계적 측정, Stage 2 비회귀
  케이스 사전 점검.
- 중첩의 중첩(2단계 이상) / rowspan 걸친 중첩행 / TAC 중첩 표 경계 → Stage 2 에서 범위 명시
  (1단계 중첩 우선, 그 외는 기존 atom 폴백 유지).
- 부분 행(한 중첩행이 페이지보다 큼) 발생 시 한컴 동작 확인 — 우선 행 경계까지만 분할(부분 행
  미분할) 후 필요 시 확장.
