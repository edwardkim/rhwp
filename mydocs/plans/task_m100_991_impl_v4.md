# 구현계획서 v4 — 글자처럼취급 표 분할 금지 (3단계 추가)

- 타스크: 로컬 task991
- 브랜치: `local/task991` (#990 위로 재구성 완료)
- 선행: `task_m100_991_impl_v3.md` (2단계 분할 셀 줄 중복 — 완료)
- 작성일: 2026-05-19

## 배경

작업지시자 확인: 6쪽 하단 ☞ 표("노후 시스템 운영 환경…")가 6→7쪽으로 분할되는데, 한컴 2022 PDF는 이 표를 분할하지 않고 7쪽에 통째로 배치한다.

## 문제

`treat_as_char=true`(글자처럼 취급) 표는 한 글자처럼 페이지 경계에서 분할되지 않고 통째로 이동해야 한다. 그러나 표 조판 분기(`typeset.rs:2060`)는 `ft.is_tac`로 갈리고:

- `ft.is_tac=true` → `typeset_tac_table` — 분할 안 함, 통째 이동 (정상)
- `ft.is_tac=false` → `typeset_block_table` — fits 실패 시 행 단위 분할

☞ 표는 `treat_as_char=true`지만 **빈 문단 + 표 1개**라 `is_tac_table_inline`이 `false` → `ft.is_tac=false` → `typeset_block_table`로 가서 **분할**된다. "글자처럼 취급"인데 인라인 판정을 못 받아 일반 블록 표처럼 잘린다.

(분할되는 다른 표 pi=525·pi=124는 모두 `treat_as_char=false` — 정상 분할 대상. 미사용 코드 `engine.rs::paginate_table_control`에는 동일한 "글자처럼취급 표 통째 이동" 분기가 이미 있으나, 실사용 경로 `typeset.rs`에 누락됨.)

## 수정

`typeset_block_table` 의 fits 검사 실패 직후, `treat_as_char` 표 가드 추가:

```rust
// fits 검사
if st.current_height + table_total <= available {
    place_table_with_text(...); return;
}
// [Task #991] treat_as_char 표는 페이지 경계에서 분할하지 않는다 —
// 한 글자처럼 통째로 다음 페이지/단으로 이동(typeset_tac_table 정책 정합).
// 한 페이지보다 큰 표는 분할 외 방법이 없으므로 기존 분할 로직으로 폴백.
if table.common.treat_as_char && table_total <= available {
    if !st.current_items.is_empty() { st.advance_column_or_new_page(); }
    place_table_with_text(st, para_idx, ctrl_idx, para, table, fmt, table_total);
    return;
}
// 이하 기존 분할 로직 (treat_as_char=false 또는 페이지 초과 표)
```

- `table_total <= available` 가드: 빈 페이지에도 안 들어가는 초대형 표는 분할 폴백(무한 루프 방지).
- `place_table_with_text` 호출 인자는 fits 분기와 동일.

## 단계

### 3단계 — 글자처럼취급 표 분할 금지

- 위 가드 구현.
- 검증: ☞ 표가 7쪽에 통째 배치(6쪽 미분할)되는지 `dump-pages`/SVG 확인. 한컴 PDF 정합.
- `cargo test` 전체 + `cargo clippy` 무경고.
- 골든 SVG 회귀 없음. 비공개 샘플 180쪽 전수 — 수정 전후 변경 쪽을 PDF와 대조.
- 산출물: `task_m100_991_stage3.md` + 커밋.

### 4단계 — 최종 검증·보고

- WASM 재빌드 영향 확인.
- `task_m100_991_stage4.md` + `report/task_m100_991_report.md` + `orders/20260519.md`.

## 범위

- 포함: `typeset_block_table` 의 `treat_as_char` 표 분할 금지.
- 제외: 페이지 수 ±1 드리프트, HWP3, 파서.
- 비공개 HWPX/PDF 미커밋.
