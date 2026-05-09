# Task #713 Stage 1 (RED) 완료 보고서

**Issue**: [#713](https://github.com/edwardkim/rhwp/issues/713)
**Stage**: 1 — TDD RED
**작성일**: 2026-05-08

---

## 산출물

- **신규 회귀 테스트**: `tests/issue_713.rs`
- **단언**:
  1. `pi=586 ci=0` 표의 row 8 셀들이 **단일 페이지**에만 등장 (분할 등장 0)
  2. row 8 셀의 `clip` 플래그가 **모두 false** (인트라-로우 분할 클리핑 0)

## 테스트 실행 결과 (RED — 의도된 FAIL)

```
$ cargo test --test issue_713 -- --nocapture
[issue_713] page_count=40 row 8 cells found across 10 (page, clip) entries
[issue_713] row 8 cells appear on pages={35, 36} clipped_cells=10

panicked at tests/issue_713.rs:96:
RowBreak 표 행 8 가 2 페이지에 분할 등장: pages={35, 36}
test issue_713_rowbreak_table_no_intra_row_split ... FAILED
```

→ 결함 정확 검출:
- row 8 (5개 셀) 이 페이지 35, 36 양쪽에 등장 → 10 entries
- 10 셀 모두 `clip=true` → 인트라-로우 분할 명세 위반

## 베이스라인 환경

- 브랜치: `local/task713` (stream/devel = 2fe386c4 베이스, Task #643/#712 미적용)
- page_count = 40 (Task #643 미적용 상태)
- 결함 페이지 인덱스: 35, 36 (= page 36, 37)

## 다음 단계 (Stage 2 — 분석)

1. `RHWP_TASK713_DEBUG=1` 트레이스 추가 (`typeset.rs` 인트라-로우 분할 분기 3 위치)
2. row 8 진입 시 `next_can_intra_split=true`, `mt.page_break=RowBreak`, `split_end_limit=17.6` 트레이스 확인
3. 가설 H1 (page_break 모드 가드) 의 위치/패치 확정
4. 보고서 + Stage 3 GREEN 진입

## 승인 요청

Stage 1 RED 완료. Stage 2 (분석) 진행 승인 요청.
