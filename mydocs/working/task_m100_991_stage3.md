# 3단계 보고서 — 1행 글자처럼취급 표 분할 금지

- 타스크: 로컬 task991
- 단계: 3/4
- 구현계획서: `task_m100_991_impl_v4.md`
- 작성일: 2026-05-19

## 1. 문제

6쪽 하단 ☞ 표(섹션 0 pi=103, 1행×1열, `treat_as_char=true`)가 6→7쪽으로 분할되나, 한컴 2022 PDF는 이 표를 7쪽에 통째로 배치한다.

## 2. 원인

표 조판 분기(`typeset.rs:2060`)는 `ft.is_tac`로 갈린다. ☞ 표는 `treat_as_char=true`지만 빈 문단 단독 앵커라 `is_tac_table_inline=false` → `ft.is_tac=false` → `typeset_block_table`로 가서 행 분할 로직을 탄다. 1행 표는 행 경계가 없어 인트라-셀(셀 내용 중간 절단) 분할만 가능한데, 글자처럼취급 표는 본문 흐름 안의 인라인 개체이므로 인트라-셀 분할이 부적절하다.

## 3. 1차 시도와 정정

`typeset_block_table` 에서 **모든** `treat_as_char` 표를 통째 이동시켰더니 페이지 수가 180→**187**로 급증했다. 원인: 본 문서에는 6×3·7×3 형태의 다행(多行) 글자처럼취급 표(SFR 요구사항 표 등)가 다수 있고, 이들은 **행 경계 분할이 정상**인데 통째 이동시켜 페이지 꼬리 공간이 대량 낭비됐다.

분할 경로 진입 표를 전수 조사한 결과 `treat_as_char && 1행`은 **단 2개**(pi=103, pi=417)뿐임을 확인. 가드를 **1행 글자처럼취급 표**로 좁혔다.

## 4. 수정

`src/renderer/typeset.rs` `typeset_block_table` — fits 검사 실패 직후:

```rust
if table.common.treat_as_char && table.row_count <= 1 && table_total <= available {
    if !st.current_items.is_empty() { st.advance_column_or_new_page(); }
    place_table_with_text(...);  // 통째 배치
    return;
}
```

- 1행 tac 표만 대상 → 다행 tac 표(행 경계 분할)는 불변.
- `table_total <= available` 가드 → 한 페이지보다 큰 표는 분할 폴백(무한 루프 방지).

## 5. 검증

- ☞ 표(pi=103): 6쪽 미배치, 7쪽에 `Table`(분할 아님)로 통째 배치. SVG 렌더 — 6쪽은 "당면과제" 절까지, 7쪽은 ☞ 5개 항목 박스 + "Ⅲ 사업추진 방안". **한컴 PDF 7쪽과 정합.**
- `cargo test --release` 전체 통과 — **1482 passed, 0 failed**.
- `cargo clippy --release` 경고 0.
- 문서 본문 텍스트 보존 — 180쪽본 96553자 vs 181쪽본 96560자(+7자 = 추가 1쪽 꼬리말·쪽번호 자릿수 차이, 본문 손실·중복 없음).

## 6. 페이지 수 영향

180 → 181쪽(+1). 표를 분할하지 않으면 6쪽 꼬리 공간(약 84px)이 비므로 +1쪽은 불가피한 결과다. 한컴(179쪽)은 동일하게 표를 통째 배치하면서도 전반의 미세 정밀도가 더 촘촘해 179쪽을 유지한다 — 이 ±쪽 누적 드리프트는 본 타스크 범위 밖의 별도 사안이다. 표 배치 자체는 한컴과 정합한다.

## 7. 다음 단계

4단계: 최종 결과보고서 + WASM 재빌드 영향 확인 + `orders/` 갱신.
