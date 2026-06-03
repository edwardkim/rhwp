# Stage 2 설계보고서 — #1025: 컷 모델 rowspan 확장 (page-larger 셀 내부 분할)

- 타스크: #1025 / 브랜치 `local/task1025` (base edf62865)
- 작성일: 2026-05-20
- 단계: Stage 2 — 셀 내부 분할 설계 (코드 무수정). Stage 3 구현 전 승인 게이트.
- 선행: Stage 1(`..._stage1.md`) — 원인·한컴 PDF 정답 확정.

## 1. 현 컷 모델 (Stage 1 확정)

- `RowCut = Vec<usize>`: 행의 `row_span==1` 셀(col 오름차순)별 소비 유닛 수.
- `advance_row_cut(row, start_cut, avail)`: `row_span==1` 셀만 순회, CellUnit(합성 줄/
  중첩표 atom) 단위로 avail 까지 진행 → `RowCutResult{end_cut, hit_hard_break, …}`.
- `row_cut_content_height` / `cell_line_ranges_from_cut`: 동일하게 `row_span==1` 셀 전제.
- typeset walk(`typeset.rs:2762~`): 행별 누적. **`rowspan_touched[r]` 행은
  `cut_row_h[r]=mt.row_heights[r]` 원자 배치**(2785) → page 시작이면 강제(overflow).
  `protected` rowspan 블록(`row_block_for`, block_size 2..BLOCK_UNIT_MAX)도 원자.

**결함**: rowspan 셀이 걸친 행의 `row_span==1` 거대 셀(pi=324 cell[10] 1024px)이 분할 불가.

## 2. 한컴 정답 (Stage 1) 재확인

1. 거대 내용 셀 = **문단 경계 분할**.
2. 헤더 행 **반복**(기존 repeat_header).
3. rs>1 라벨 셀(상세설명/세부내용) = **첫 조각에만 표시, 연속 조각 공란**.

## 3. 설계 — 행블록(row-block) 컷

### 3.1 행블록 정의
`row_block`(start_row..end_row) = rs>1 셀로 묶인 연속 행 집합(`mt.row_block_for` 활용).
블록 셀 = (a) 블록을 걸친 rs>1 셀 + (b) 블록 내 각 행의 `row_span==1` 셀.
**안정 순서**: (row, col) 오름차순 — 컷 벡터 인덱스 기준.

### 3.2 블록 컷 함수 (advance_row_cut 일반화)
`advance_row_block_cut(table, b_start, b_end, start_cut, avail, styles) -> RowCutResult`:
- 블록 셀 전부(rs>1 포함) 순회, 각 셀 CellUnit 을 avail 까지 진행.
- **rs>1 셀(라벨)**: 작은 콘텐츠 → 첫 조각(start_cut 비었을 때)에 전량 소비. 연속 조각
  (start_cut 이 이미 전량)에선 0 유닛 → **렌더 공란**(정답 #3).
- **거대 `row_span==1` 셀**: CellUnit(문단 줄) 단위로 page 경계까지 채우고 잔여는
  다음 조각 — `end_cut[그 셀]` 에 기록.
- `consumed_height` = 블록 셀별 표시 높이 max(rs>1 셀 포함; 단 연속 조각의 rs>1 셀은 0).
- 기존 `advance_row_cut` 은 단일 행(block_size 1)일 때와 동일 결과(회귀 0 목표).

### 3.3 높이·줄범위 함수 확장
- `row_cut_content_height` → `row_block_content_height(b_start,b_end,start_cut,end_cut)`:
  블록 셀별 (content_in_cut + pad), 행 max → 블록 표시 높이.
- `cell_line_ranges_from_cut`: 셀 단위라 변경 불필요(블록 셀에 그대로 적용).

### 3.4 typeset walk 변경 (`typeset.rs:2785` rowspan_touched 분기)
- 현재: `rowspan_touched[r]` → 원자 `cut_row_h[r]`.
- 변경: 블록을 `advance_row_block_cut` 으로 진행. 블록이 avail 초과 & 분할 가능(거대
  `row_span==1` 셀 존재)이면 블록 내부 컷 분할. `split_end_cut` 에 블록 셀 컷 기록.
- `protected` 블록(2770): 거대 셀 미포함이면 기존 원자 유지; 거대 셀 포함이면 분할 허용.

### 3.5 렌더러 정합 (`table_partial.rs`)
- 블록 컷의 셀별 줄 범위(`cell_line_ranges_from_cut`)로 그림.
- rs>1 셀: 첫 조각만 콘텐츠, 연속 조각 공란(셀 박스는 그리되 내부 비움) — 정답 #3.
- 헤더 반복은 기존 경로 유지.

## 4. 비범위·원자 유지
- **중첩표 보유 셀**(pi=323 cell 내부표): CellUnit 의 중첩 atom 은 분할하지 않음(atom 단위
  유지). atom 1개가 page-larger 면 그 atom 은 통째(잔여 overflow 허용) — 별도 후속.
- 표 외 page-larger 문단/그림(pi=567). WASM.

## 5. 스키마/함수 변경 요약 (Stage 3 대상)
| 함수 | 변경 |
|------|------|
| `advance_row_cut` | `advance_row_block_cut`(블록 셀, rs>1 포함)로 일반화. 단일행 호환. |
| `row_cut_content_height` | 블록 버전 추가(또는 b_start==b_end 호환). |
| `RowCut` 인덱싱 | 블록 셀 (row,col) 안정 순서로 정의(단일행=col 순서와 동일). |
| `typeset.rs` walk | rowspan_touched 분기를 블록 컷 분할로 교체. |
| `table_partial.rs` | 블록 컷 렌더 + rs>1 셀 연속조각 공란. |

## 6. 리스크·완화
- **최고 위험**: #993/#1022 분할 전반 영향(단일행 컷 회귀). 완화: 단일행=블록 호환 보장 +
  각 Stage 골든·`svg_snapshot`·LAYOUT_OVERFLOW 게이트. 단일행 결과 불변 단위테스트.
- rs>1 셀 시각 분할 경계(셀 박스 연속) 한컴 정합 — Stage 4 PDF 대조.
- 블록 컷 인덱스 정렬 불일치 시 렌더↔페이지네이터 어긋남 → 동일 순서 단일 정의로 고정.

## 7. 단계 진행 (impl 계획서 §단계와 동일)
Stage 3(typeset 구현) → Stage 4(table_partial 렌더) → Stage 5(검증·골든·회귀·보고).
각 Stage 소스 커밋 + 게이트.
