# Task #1748 2단계 완료 보고 — 컷 걸침 rowspan 셀 높이 기반 유닛 컷 구현

## 구현 내용

### `src/renderer/layout/table_layout.rs`

- `cell_units_fitting_height(cell, table, styles, budget)` 헬퍼 추가 — 셀 유닛
  누적높이가 budget(패딩 제외 콘텐츠 예산) 안에 들어가는 선두 유닛 수 (EPS 0.1px).
  컷 페이지의 eu 와 연속 페이지의 su 가 같은 예산 식으로 계산되어 경계 유닛
  인덱스가 산술적으로 일치한다.

### `src/renderer/layout/table_partial.rs`

1. **걸침 판별** (`is_rowbreak_straddle`): 비블록 분할 + `row_span > 1` +
   RowBreak 표 + 기존 컷 부기 밖(`!is_in_split_row`) + 반복 제목행 셀 제외에서
   - 시작 걸침: `cell_row < start_row < cell_end_row` (연속 페이지)
   - 끝 걸침: span 이 프래그먼트 끝을 넘거나(`cell_end_row > render_range_end`)
     마지막 span 행이 분할 행(`cell_end_row == render_range_end && !end_cut.is_empty()`)
2. **높이 기반 유닛 컷**: 시작 걸침의 이전 소비 높이(prior_h)는 2b 행높이
   오버라이드와 동일한 식으로 재계산 — 온전 행 `row_cut_content_height(r, [], [])`
   (0 이면 resolve 값), 분할 행 `row_cut_content_height(start_row, [], start_cut)`.
   `su = fit(prior_h − pad_top)`, `eu = 끝 걸침 ? fit(prior_h + cell_h − pad_top) : MAX`.
   기존 `cell_line_ranges_from_cut` 로 줄 범위 변환 (기존 분할 셀과 동일 경로).
3. `clip: is_in_split_row || is_rowbreak_straddle`, Top 정렬 강제 조건에도 동일 반영.
4. `resolved_row_heights`(2b 오버라이드 이전 원본) 클론 보관 — prior_h 재계산용.

페이지네이션(typeset.rs / advance_row_cut)은 **불변** (p6 회계 981.1 ≤ 988.3 정상이었음).

### 신규 테스트 `tests/issue_1748_rowbreak_straddle_rowspan.rs` (3건)

| 테스트 | 가드 | 수정 전 실측 |
|--------|------|--------------|
| `cut_page_straddling_rowspan_cell_text_stays_inside_cells` | p6 셀 하단 초과 TextLine 없음 | +44.9px 초과 |
| `cut_page_no_text_ink_below_table_fragment` | p6 표 프래그먼트 아래 텍스트 잉크 없음 | 1122.5 vs 1077.6 |
| `continuation_page_straddling_rowspan_cell_text_stays_inside_cells` | p7 중복 재렌더 없음 | +61.9px 초과 |

## 검증 결과

### 본 건 (96 DPI dark-pixel bbox, pymupdf, vs 한글 2024 PDF)

| 페이지 | 수정 전 | 수정 후 | 게이트 |
|--------|--------:|--------:|--------|
| p6 Δbot | **+13** | **−4** | ≤ ~5px ✅ |
| p6 Δtop | −1 | −1 | 불변 ✅ |
| p5 Δtop/Δbot | −1/−6 | −1/−6 | 무회귀 ✅ |

- 전체 53쪽 불변 (페이지네이션 무변경).
- p7 이어그리기: 한글 p7과 같은 잔여 줄 렌더("콘크리트 1일 타설량이 120 /
  세제곱미터 이상인 경우 / : 120세제곱미터마다"), 중복·아래 행 침범 해소.
  (rhwp 는 p5 시점부터 1행 선행하는 별개 표류(#1772/#1774 계열)가 있어 줄 단위
  완전 일치는 본 타스크 범위 밖 — 기하 게이트는 충족.)

### 회귀 게이트

| 게이트 | 결과 |
|--------|------|
| giant (`samples/task1718/table_giant_cell_overfill.hwp`) | 42쪽 불변, p5/p6 Δtop −3 / Δbot −2 (#1728 v2 의 −4 수준 유지) ✅ |
| byeolpyo1 / byeolpyo4 | 4쪽 / 26쪽 무회귀 (#1718 이후 26 이 현행 기준) ✅ |
| `cargo test --release` 전체 | **2748 passed / 실질 실패 0** ✅ |
| svg_snapshot 골든 | 내용 기준 8/8 불변 ✅ (아래 참고) |
| `rustfmt --check` (변경 파일) | 통과 (newline 경고는 autocrlf 환경 노이즈) |

**참고 — svg_snapshot 7건 "실패"는 Windows 환경 노이즈**: `core.autocrlf=true`
체크아웃이 골든 SVG 를 CRLF 로 변환해 LF 생성물과 바이트 불일치. CR 제거 비교 시
7건 모두 diff 0줄 (CI Linux 무영향). #1775(CFB 경로 구분자)와 같은 부류의 Windows
개발환경 이슈 — 별도 이슈 등록 예정.

## 다음 단계

3단계 — 잔여 게이트(클리핑 게이트, 캐스케이드 후반부 페이지 대조) + 최종 보고.
