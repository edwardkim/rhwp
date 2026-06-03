# Task #398 Stage 2 — 분할 호출부에 블록 정책 적용

## 발견 사항

`split_table_rows` 와 동일한 분할 로직이 **두 곳에 중복 존재**한다는 사실을 작업 중 확인:

1. `src/renderer/pagination/engine.rs::split_table_rows` (1445~)
2. `src/renderer/typeset.rs` 내 `paginate_table` 계열 (1378~) — **실제 SVG 내보내기 경로가 사용**

엔진 한쪽만 수정 시 회귀 검증에서 변화 없음을 발견하여 두 파일 모두 동일 패턴으로 수정.

## 변경 사항

### `src/renderer/pagination/engine.rs::split_table_rows`

1. **Pre-loop 분할 판정 (line 1492~)**: `first_row_h = mt.row_heights[0]` → `first_block_h = mt.row_block_for(0).2` 로 교체.
   - `first_block_is_single_row` 플래그 추가.
   - 인트라-로우 분할은 단일 행 블록에서만 활성.

2. **Loop body 분할 결정 (line ~1606)**:
   - `find_break_row` 결과에 `snap_to_block_boundary` 적용 → rowspan 묶음 중간 분할 차단.
   - `cursor_row` 가 속한 블록 (`cur_b_*`) / 분할 후보 행 `r` 의 블록 (`next_b_*`) 단일 여부 검사.
   - 인트라-로우 분할 / 강제 split 모두 단일 행 블록에서만 시도.
   - 다중 행 블록이 들어가지 않으면 `end_row = cur_b_end` (블록 전체 한 단위로 배치).

### `src/renderer/typeset.rs::paginate_table` (실제 호출 경로)

위와 동일한 패턴 적용 (변수 이름·구조 동일).

### 디버그 출력 정리

조사 중 추가했던 `eprintln!("DBG_T398 ...")` 제거.

## 검증

### 본 샘플 (Task #398 회귀)

`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx` 재내보내기:

**페이지 1 (`dump-pages -p 0`)** — 표 사라짐:
```
... pi=21 (빈) vpos=66830  (마지막 항목)
```

**페이지 2 (`dump-pages -p 1`)**:
```
Table  pi=22 ci=0  3x3  635.0x924.5px  wrap=TopAndBottom tac=false  vpos=68590
```
→ PartialTable 분할 없이 표 전체가 페이지 2에 시작 (PDF/한글과 동일).

**페이지 1 SVG 텍스트 (제목 부재 확인)**:
| 글자 | 페이지 1 | 페이지 2 |
|------|----------|----------|
| 분 | 0 | 1 |
| 석 | 0 | 1 |
| 답 | 0 | 1 |
| 례 | 0 | 1 |

(페이지 1의 "기:1 보:2" 는 "본 보고서는...주시기 바랍니다." 본문 텍스트.)

### 전체 단위 테스트

```
cargo test --lib
  → 1023 passed; 0 failed; 1 ignored
```

기존 표 분할 테스트(`pagination/tests.rs::test_typeset_page_break` 외 17개) + 신규 단계 1 테스트 7개 모두 통과.

## 다음 단계

- 단계 3: 골든 샘플 회귀 검증 (`re_sample_gen` 또는 수동 SVG diff) + 최종 보고서 + orders 갱신.
