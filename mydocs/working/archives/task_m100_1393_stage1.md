# Task M100 #1393 — 1단계 완료 보고서 (게이트 동승)

- 브랜치: `local/task1393`
- 작성일: 2026-06-14
- 수정 파일: `src/serializer/hwpx/roundtrip.rs`

## 1. 구현 내용

### 1.1 variant + Display

- `IrDifference::TablePageBreak { section, paragraph, path, detail }` + Display
  (`…tbl page_break: expected={:?} actual={:?}`).

### 1.2 비교 추가

- `diff_paragraph_char_shapes`의 Table arm에 `ta.page_break != tb.page_break` 비교
  1지점 추가 (셀 순회 전). path `…/ctrl[{ci}]tbl`. 셀 문단 재귀가 중첩 표까지
  도달하므로 최상위·중첩 표 모두 동승.
- 방출은 무변경 (PR #1405 `ad55059f` 정정 — 본 게이트는 회귀 봉인용).

## 2. 단위 테스트

| 테스트 | 검증 |
|--------|------|
| `task1393_table_page_break_diff_in_gate` | RowBreak vs CellBreak → `TablePageBreak` 검출 + path·detail 고정 |
| `task1393_form_002_page_break_roundtrips` | 실샘플 form-002 roundtrip 게이트 0 (CELL 보존) |

`cargo test --lib serializer::hwpx::roundtrip` 48 passed.

## 3. 전수 검증

- `cargo test --test hwpx_roundtrip_baseline` 4 passed — **B=0 유지, 신규 xfail 0**.
- `hwpx-roundtrip --batch samples/hwpx`: PASS 53 / **IR_DIFF 0** (page_break 게이트
  동승 후에도 차이 0 — 전수 보존 확인) / SERIALIZE_FAIL 0 / PARSE_FAIL 1(제외).
- `cargo fmt --check` 통과.

## 4. 다음 단계

2단계 — 매뉴얼 + CI급(release-test) + 최종 보고서 (방출=PR #1405 / 게이트=본 타스크
역할 분리 기록).

승인 요청드립니다.
