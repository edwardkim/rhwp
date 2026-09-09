# Task #1663 결과보고서 — 빈 host co-anchored 자리차지 float 표 orphan control + 표 뒤 말미 빈 문단 흡수

## 이슈

- GitHub #1663: 빈 host 문단에 co-anchored 자리차지(TopAndBottom, vert=문단) RowBreak 표가 둘 이상이고 선행 표가 페이지를 채운 경우, 후속 표가 한 페이지에 들어가는데도 행 분할되어 머리 행만 앞 페이지에 남고(orphan) 본문이 다음 페이지로 분리. 추가로 그 표 뒤 말미 빈 문단이 빈 페이지를 만든다.
- 관련: #1510/#1535/#1549/#1639(co-anchored float 표 계열). 본 건은 페이지네이션(분할 vs 통째 이월) 영역.

## 근본 원인 (소스 확정)

`src/renderer/typeset.rs`:

- `typeset_block_table`: 자리차지 표가 현재 페이지 잔여 초과 시 행 단위(RowBreak) 분할로 진입. 선행 co-anchored float 가 채운 후속 표도 `table_total <= available`(한 페이지에 들어감)인데 분할.
- `typeset_section_with_variant`: 말미 빈 문단이 현재 page 잔여 초과 시 새 page 생성. 기존 빈페이지 차단 가드는 "다음 문단이 reset/쪽나눔"만 커버하고 문서 끝 trailing 빈 문단은 미커버.

한컴은 한 페이지에 들어가는 자리차지 표를 분할하지 않고 통째로 다음 페이지에 두며, 그 뒤 빈 문단을 trailing overflow 로 흡수한다.

## 수정 (`src/renderer/typeset.rs`)

1. **orphan control** (`typeset_block_table`, fits-whole 분기 직전): 다음을 모두 만족하면 `advance_column_or_new_page` 로 표 통째 이월.
   - `is_para_topbottom_float(table.common)` && `page_break == RowBreak`
   - `has_preceding_coanchored_float` — 같은 host 문단의 앞선 컨트롤에 다른 자리차지 float 표 존재(= co-anchored 그룹의 2번째 이후)
   - `!current_items.is_empty()` && `current_height + table_total > available`(잔여 부족) && `table_total <= available`(fresh 적합)
2. **trailing 빈 문단 흡수** (`typeset_section_with_variant`, 문단 루프 pre-placement): 빈 문단(텍스트·컨트롤 없음)이 `last_content_para_idx` 뒤(trailing)이고, page 마지막 항목이 co-anchored 자리차지 표이며, 명시적 쪽나누기가 없고, 현재 page 잔여를 초과하면 `continue`(빈 page 미생성).

**경계/한정**: 단독 anchored 표(host 의 첫/유일 float)·일반 문단 흐름은 불변. `table_total <= available` 로 페이지보다 큰 표는 통째 이월 안 함(무한 push 차단). co-anchored 신호로 trailing 흡수를 한정해 편집 동작(Enter 누적)·본문 trailing 빈 줄 페이지네이션 보존.

## 산출물

- 수정: `src/renderer/typeset.rs` (가드 2곳 +88줄, co-anchored 한정 주석 포함)
- 회귀 fixture(issue1639 Clone and Narrow): `samples/issue1663_coanchored_float_orphan.hwpx` — 빈 host + co-anchored 자리차지 RowBreak 표 A(2행≈800px)·B(21행≈784px) 양수 offset + 말미 빈 문단 3개. 합성 셀 텍스트만(실문서 미포함).
- 회귀 테스트: `tests/issue_1663.rs` (2 tests) — (1) 후속 표 B 가 page0 에 행 분할되지 않고 page1 에 통째(`build_page_render_tree`로 ci=2 page0/ci=3 page1·non-page0 단언), (2) 말미 빈 문단 흡수로 `page_count() == 2`.

## 검증

> 본 변경은 최신 `devel`(#1722/#1686 머지 이후, `063383b9`) 기준으로 rebase 후 재검증했다. `typeset.rs`
> 에서 #1686/#1722 와 1곳 충돌 — orphan pre-check(본 수정)와 #1722 의 overlay-shapes fit 확장은
> 직교라 순차 병합. #1722 는 실양식의 trailing 빈 페이지는 없앴으나 orphan(헤더-본문 분리)은 미해결이라
> 본 point-fix 가 여전히 유효함을 실양식·합성 fixture 로 재확인.

- 전체 `cargo test`: 168 바이너리 **2745 passed / 0 failed / 23 ignored**(로컬 미보유 corpus 등). 관련 계열 전부 `ok` — `issue_1686`(4, **#1722 무회귀**)·`issue_1510`(4)·`issue_1535`(1)·`issue_1549`(2)·`issue_1639`(3)·신규 `issue_1663`(2).
- `cargo fmt --all -- --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo check --target wasm32-unknown-unknown --lib` 통과.
- fixture: clean 바이너리 3페이지(p0 A+PartialTable B, p1 B 잔여, p2 빈 page = 두 버그 재현) → fixed 바이너리 2페이지(p0 A, p1 B 통째, 빈 page 없음).
- 시각/페이지수(참고 — 한컴 PDF 는 정답지 아님): 최신 `devel` 에서 실양식(검증점검표)은 **2페이지(orphan 잔존) → 2페이지(orphan 해소, 머리행+본문 동일 페이지)**, 합성 fixture 는 3→2. 나린뜰 8 양식 전부 한컴 페이지수 일치(무회귀), 복잡 문서 `hwpx_sample2` 29 유지.
- 교차검증: render tree(`build_page_render_tree`)·typeset(`dump-pages`) 양 경로 일관.

## 비고

- 옵션 검토: A(keep-block, magic 상수·회귀면 大) 기각, B(fits-fresh 통째 이월 + co-anchored 한정) 채택. 단독 anchored 표까지 이월하면 `hwpx_sample2` 29→33 회귀(clean/orphan-only `dump-pages` 대조로 pi=32/77/100/127 `ci=0` 과도 이월 확인) → co-anchored 한정 필요.
- trailing 빈 문단 흡수도 광범위 적용 시 `test_page_overflow_with_enter`·`hwpx_sample2`(29) 회귀 → page 마지막이 co-anchored 자리차지 표일 때로 한정.
- 한정 전제: `has_preceding_coanchored_float` 는 host 컨트롤 *배열 순서*로 선행 float 존재를 판정한다. 빈-host 양수 전용 voffset 재정렬(#986/#1088)로 배열 순서 ≠ 배치 순서가 되는 드문 경우 orphan 가드가 의도한 표를 놓칠 수 있다. 대상 케이스(음수 offset 혼재 → 정렬 OFF, 배열=배치)와 검증한 8 양식·`hwpx_sample2` 는 모두 배열=배치라 무영향. 필요 시 배치 순서(`current_items` 의 선행 동일-host float) 기준으로 확장 여지.
- 별개 이슈(범위 외): 첫 자리차지 표 큰 셀(valign=Center)의 세로 위치가 한컴과 다름(셀 세로정렬/높이 렌더 차이). 본 변경과 무관(수정 전후 동일).
- 비공개 실문서는 fixture 로 커밋하지 않고 로컬 대조에만 사용. float = 페이지네이션 인접 영역, 메인테이너 재검증 대상.
