# Task #398 — 표 페이지 분할 시 rowspan>1 셀 누락 (최종 보고서)

## 문제 요약

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 1쪽 하단에 다음 표(pi=22, 3×3, 제목 "<2025년 기부·답례품 실적 분석보고서 요약>")의 첫 행이 잘려 들어가고 rowspan=2 셀의 제목 텍스트가 본문 영역 끝에 겹쳐서 그려짐. PDF 원본은 표 전체를 2쪽으로 미룸.

## 근본 원인

표 페이지 분할 알고리즘이 분할 단위를 **개별 행**으로 산정하면서 **rowspan>1 셀의 시각적 점유 영역을 누락**.

- `mt.row_heights[r]` 산출 시 `cell.row_span == 1` 셀만 집계 (`src/renderer/height_measurer.rs:466`).
- `split_table_rows` 의 첫 행 판정 (`src/renderer/pagination/engine.rs:1492`, `src/renderer/typeset.rs:1384`) 이 `row_heights[0]` 만 사용 → rowspan=2 셀(35.4px)을 17.27px로 과소평가.
- 잔여 공간 19px ≥ 17.27px → 행 0을 페이지 1에 배치 → rowspan 셀이 19px 안에 들어가지 못해 시각적으로 잘려 그려짐.

## 해결책

분할 가능 단위를 "행" → "rowspan 묶음 블록"으로 확장.

### 데이터 구조 (단계 1)

`MeasuredTable` 에 사전 계산된 블록 경계 캐시 추가:
- `row_block_start[r]`: r 행을 포함하는 모든 rowspan 셀의 최소 시작 행
- `row_block_end[r]`: r 행을 포함하는 모든 rowspan 셀의 최대 종료 행 (exclusive)

산출은 `compute_row_blocks(table, row_count)` 헬퍼에서 모든 셀의 `(row, row_span)` 을 검사하여 전이 폐포로 통합. `mt.cells` (`page_break == CellBreak` 일 때만 채움) 에 의존하지 않고 모델의 `table.cells` 에서 직접 계산하여 모든 page_break 모드에 일관 적용.

신규 메서드:
- `MeasuredTable::row_block_for(row) -> (start, end_exclusive, height)`
- `MeasuredTable::snap_to_block_boundary(end_row) -> usize`

### 분할 호출부 (단계 2)

두 곳의 분할 로직(중복):
- `src/renderer/pagination/engine.rs::split_table_rows`
- `src/renderer/typeset.rs::paginate_table` (실제 SVG 내보내기 경로)

변경:
1. **Pre-loop 분할 판정**: `first_row_h = mt.row_heights[0]` → `first_block_h = mt.row_block_for(0).2`.
2. **Loop body**:
   - `find_break_row` 결과에 `snap_to_block_boundary` 적용 → rowspan 묶음 중간 분할 차단.
   - 인트라-로우 분할 / 강제 split 모두 단일 행 블록(`block_end == block_start + 1`)에서만 시도.
   - 다중 행 블록이 들어가지 않으면 `end_row = cur_b_end` (블록 전체 한 단위 배치, 페이지 초과 가능).

## 검증 결과

### 본 샘플 (Task #398 회귀)

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 재내보내기:

**페이지 1 (수정 전 → 수정 후)**:
```
이전: ... pi=21 (빈) → PartialTable pi=22 ci=0 rows=0..1 cont=false 3x3
이후: ... pi=21 (빈)  (마지막 항목, 표 사라짐)
```

**페이지 2 (수정 전 → 수정 후)**:
```
이전: PartialTable pi=22 ci=0 rows=1..3 cont=true
이후: Table        pi=22 ci=0 3x3 635.0x924.5px wrap=TopAndBottom tac=false vpos=68590
```

→ PartialTable 분할 사라지고 표 전체가 페이지 2에 시작 (PDF/한글과 동일).

**페이지 1 SVG 텍스트**:
| 글자 | 페이지 1 | 페이지 2 |
|------|----------|----------|
| 분 | 0 | 1 |
| 석 | 0 | 1 |
| 답 | 0 | 1 |
| 례 | 0 | 1 |

### 회귀 테스트

| 스위트 | 결과 |
|--------|------|
| `cargo test --lib` | **1023 passed**; 0 failed |
| `cargo test --test svg_snapshot` (골든 6건) | **6 passed** (table-text, issue-147, issue-157, issue-267, form-002, deterministic) |
| `cargo test --tests` (전체 통합) | **1073 passed** (lib 1023 + integration 50); 0 failed |
| `cargo build --release` | 성공 |
| `samples/table-vpos-01.hwpx` 내보내기 | 5쪽 정상 |
| `samples/표-텍스트.hwpx` 내보내기 | 1쪽 정상 |

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/height_measurer.rs` | `MeasuredTable.row_block_start/end`, `compute_row_blocks` 헬퍼, `row_block_for`/`snap_to_block_boundary` 메서드, 단위 테스트 7건 |
| `src/renderer/pagination/engine.rs::split_table_rows` | pre-loop `first_block_h`, snap, 단일성 가드 |
| `src/renderer/typeset.rs::paginate_table` | 동일 패턴 적용 (실제 내보내기 경로) |
| `samples/2025년 기부·답례품 실적 지자체 보고서_양식.{hwpx,pdf}` | 신규 (회귀 검증 자료) |
| `mydocs/plans/task_m100_398.md` | 수행계획서 |
| `mydocs/plans/task_m100_398_impl.md` | 구현계획서 |
| `mydocs/working/task_m100_398_stage{1,2}.md` | 단계별 보고서 |

## 단계별 진행

| 단계 | 커밋 | 내용 |
|------|------|------|
| 1 | `4454573` | `MeasuredTable` rowspan 메타데이터 + 헬퍼 + 단위 테스트 7건 + 샘플 동반 |
| 2 | `d570ab7` | 분할 호출부에 블록 정책 적용 (engine.rs + typeset.rs) |
| 3 | (이번 커밋) | 회귀 검증 + 최종 보고서 + orders 갱신 |

## 부수 발견 사항

- 분할 로직이 `pagination/engine.rs` 와 `typeset.rs` 두 곳에 **중복**되어 있다. 향후 한쪽으로 통합 또는 공유 헬퍼화 고려 필요 (별도 타스크).
- 7쪽 차트 중복 그림 그리기 이슈는 별개 원인 (`Control::Picture` 의 TAC 그림에 대해 `PageItem::Shape` 추가 발행 + `paragraph_layout` 인라인 렌더 이중 처리). 본 타스크 범위 외 (별도 이슈 등록 필요).

## 의도된 동작 변경

`samples/` 외 다른 문서에서 rowspan>1 셀이 페이지 경계에 걸리는 경우, 표가 통째로 다음 페이지로 밀려나면서 빈 공간이 더 커질 수 있다. 이는 한글 호환 동작이며 의도된 변경이다.
