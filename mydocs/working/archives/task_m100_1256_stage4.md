# Stage 4 보고서 — Task #1256: 단위테스트 + 정리

- 이슈: edwardkim/rhwp#1256 · 브랜치: `local/task1256`

## 1. 코드 하드닝

path-1 보정 가드를 `end_y < y_offset` → **`stored_gap_px < -0.5`(=result < y_offset)** 로 정정.
- 이유: 베이스라인 result 가 이미 y_offset 이면(컬럼 하단 등 backtrack 미발동) base 를
  건드리지 않아 **spurious base-shift(후속 overflow)** 를 차단. base 이동량도 실제 하강분
  `(y_offset - result)` 로 일치.
- 검증: 페이지9 문6→문7 = 287.0px 유지, 페이지 수·오버플로우 회귀 0.

## 2. 단위테스트 추가 (`src/renderer/height_cursor.rs`)

1. `compact_endnote_between_notes_singleline_prev_keeps_gap_and_shifts_base`
   - 단일 줄 prev(ls=1984) + 문 제목 + safe_backtrack cram 상황 → result=y_offset(7mm 유지),
     `vpos_page_base` 가 하강분만큼 음수 이동 확인.
2. `compact_endnote_between_notes_skips_natural_trailing_prev`
   - 자연 trailing(ls=180<1984) prev → injected_between_notes=false → 보정 미발동, base 무이동.
- 기존 `compact_endnote_min_gap_skips_single_line_prev`(end_y==y_offset) 도 그대로 통과.

## 3. 최종 검증

- `cargo test` 전체 **1963 passed, 0 failed** (기존 1961 + 신규 2).
- 페이지 수: 3-09 23/23, 미주사이20 24, 구분선아래20 23, 3-11 21 (불변).
- `cargo fmt`(변경 파일만) 적용.

## 4. 산출물

- 최종 결과보고서: `mydocs/report/task_m100_1256_report.md`
- 후속 이슈 #1257 등록 완료.

## 5. orders 갱신 안내

`mydocs/orders/` 는 작업지시자 관할(편집하지 않음). 본 타스크 상태 반영은 작업지시자가 수행.
