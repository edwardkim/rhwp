# Task #321 v5 구현 계획서 — pi=0 block-table drift 보정

## 사전 조사

`typeset_block_table` (`src/renderer/typeset.rs:1096`) 의 fit 분기:

```rust
if st.current_height + table_total <= available {
    self.place_table_with_text(st, para_idx, ctrl_idx, para, table, fmt, table_total);
    return;
}
```

`table_total = ft.effective_height + ft.host_spacing.before + ft.host_spacing.after`

`place_table_with_text` 가 cur_h 에 `table_total` 을 더한다 (= 우리 pi=0 advance 의
대부분 차지).

문제 패턴: `wrap=TopAndBottom` + `vert_rel_to=Paper` + body 와 겹치는(또는 body 위쪽에 위치)
표는 시각적으로 절대 좌표에 그려지므로 **block advance 가 본문 cur_h 에 반영될 필요가 없다**
(col 1+ 은 이미 `pending_body_wide_top_reserve` 로 처리됨).

## Stage 6a — 가설 H1 시안 작성 + 21_언어 단일 측정

**수정 위치**: `src/renderer/typeset.rs::typeset_block_table` (line 1118-1125 부근)

**수정 시안** (조건부 advance 보정):

```rust
let host_spacing_total = ft.host_spacing.before + ft.host_spacing.after;
let table_total = ft.effective_height + host_spacing_total;

// Task #321 v5: Paper-anchored TopAndBottom block 표는 body_wide_reserved 와 동일 패턴.
// col 0 첫 문단(=pi=0) 이고 표가 본문 영역과 다른 좌표계(Paper-anchored) 일 때,
// cur_h advance 를 표 자체로 늘리지 말고 host paragraph 의 본문 line_segs 만 반영.
let suppress_table_advance = st.current_column == 0
    && para_idx_is_first_in_col(...)  // 정확한 조건 미정 — 6a 에서 결정
    && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
    && matches!(table.common.vert_rel_to, VertRelTo::Paper)
    && !table.common.treat_as_char;
let effective_advance = if suppress_table_advance {
    fmt.total_height  // host paragraph 의 본문 라인 + spacing 만
} else {
    table_total
};

if st.current_height + effective_advance <= available {
    self.place_table_with_text(st, para_idx, ctrl_idx, para, table, fmt, effective_advance);
    return;
}
```

**검증 절차** (Stage 6a):

1. 위 시안 적용 → `cargo build --release`
2. `RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 0 ...`
3. 측정:
   - col 0 drift (pi=1 cur_h 비교) 76.3 → ~0 확인
   - dump-pages 로 col 1 첫 item = pi=10 확인
4. SVG 시각 확인 (rsvg-convert + Read 이미지) — 표 위치 무변경 + col 1 시작 "적합성..."

조건이 너무 좁거나 넓으면 6b 에서 정밀화. 너무 좁으면 다른 case 미적용, 너무 넓으면 회귀.

## Stage 6b — 회귀 검증 + 조건 정밀화

**확인 항목**:

1. `cargo test --lib`: 992 passed 유지
2. 페이지 수 비교 (`./target/release/rhwp export-svg <sample> | wc -l SVG 파일 수`):
   - 21_언어 16 (기준)
   - exam_math 20
   - exam_kor 24
   - exam_eng 10
   - form-002 등 hwpx 샘플
3. `tests/golden_svg/` 골든 diff (`cargo test golden_svg` 또는 grep 결과)
4. 21_언어 page 1 외 다른 페이지에서 동일 패턴 표 회귀 없는지 (`dump-pages -p 1..15` 비교)

회귀 발견 시 조건을 **더 좁게** (예: pi=0 만, 또는 첫 페이지만, 또는 명시적 vpos 기반 검증) 정밀화.

## Stage 6c — 보고서

**산출**:

- `mydocs/working/task_m100_321_stage6.md` — 수정 전후 측정 표, 수정 diff, 회귀 결과
- `mydocs/report/task_m100_321_v5_report.md` — 최종 결과 보고서

## 회피 사항

- ✋ 시각적 표 위치 변경 금지 (place_table_with_text 의 표 그리기 좌표는 무변경; cur_h advance 만 조정)
- ✋ col 1+ 는 이미 `pending_body_wide_top_reserve` 처리됨 — 본 수정은 col 0 한정
- ✋ TAC 표(treat_as_char=true) 는 무관 — 별도 path
- ✋ Stage 5a 에서 추가한 진단 로그는 유지 (수정 검증 도구로 활용)

## 승인 요청

본 구현 계획서 승인 후 Stage 6a 부터 순차 진행.
