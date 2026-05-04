# Task #568 구현 계획서 — 안 (a) 상세화

- **이슈**: [#568](https://github.com/edwardkim/rhwp/issues/568)
- **브랜치**: `local/task568`
- **단계**: Stage 2 (구현 계획)
- **선행 산출**: `mydocs/working/task_m100_568_stage1.md` (정밀 진단)
- **작성일**: 2026-05-04

## 1. 정정 본질 (Stage 1 결론 재진술)

`paragraph_layout.rs::layout_composed_paragraph` 의 `effective_col_x / effective_col_w` 산출(L857-866) 이 인라인 TAC 표 보유 줄의 `comp_line.segment_width` 를 사용하지 않음. 결과적으로 `available_width = col_area.width - margins` 가 과대 산출되고 Justify slack 이 선두 공백을 80 px / space 로 부풀려 인라인 표를 +175 px 우측으로 밀어버림.

## 2. 정정 안 — 안 (a)

기존 `has_picture_shape_square_wrap` 분기와 동일한 패턴(LINE_SEG.column_start/segment_width 를 effective col x/width 로 사용)을 **인라인 TAC 표 보유 줄** 에도 확장.

### 2.1 변경 위치 — `paragraph_layout.rs` L857-866

**현재 코드**:
```rust
let (effective_col_x, effective_col_w) = if has_picture_shape_square_wrap
    && comp_line.segment_width > 0
    && comp_line.segment_width < col_area_w_hu - 200
{
    let cs_px = hwpunit_to_px(comp_line.column_start, self.dpi);
    let sw_px = hwpunit_to_px(comp_line.segment_width, self.dpi);
    (col_area.x + cs_px, sw_px)
} else {
    (col_area.x, col_area.width)
};
```

**변경 후**:
```rust
// [Task #568] 인라인 TAC 표가 있는 줄도 LINE_SEG.cs/sw 적용.
// HWP 는 인라인 TAC 표가 있는 줄의 segment_width 를 표 폭 + 잔여로 좁게
// 인코딩한다 (wrap=TopAndBottom 영향). col_area.width 로 잡으면
// Justify slack 이 과대 산출되어 선두 공백이 부풀어 표를 우측으로 민다
// (exam_science.hwp 12번 응답 pi=61: +175 px 편위).
let line_has_inline_tac_table = !tac_offsets_px.is_empty() && para.map(|p| {
    let line_start = comp_line.char_start;
    let line_end = line_start + comp_line.runs.iter()
        .map(|r| r.text.chars().count()).sum::<usize>();
    tac_offsets_px.iter().any(|(pos, _, ci)| {
        *pos >= line_start && *pos <= line_end
            && matches!(p.controls.get(*ci), Some(Control::Table(t)) if t.common.treat_as_char)
    })
}).unwrap_or(false);

let (effective_col_x, effective_col_w) = if (has_picture_shape_square_wrap || line_has_inline_tac_table)
    && comp_line.segment_width > 0
    && comp_line.segment_width < col_area_w_hu - 200
{
    let cs_px = hwpunit_to_px(comp_line.column_start, self.dpi);
    let sw_px = hwpunit_to_px(comp_line.segment_width, self.dpi);
    (col_area.x + cs_px, sw_px)
} else {
    (col_area.x, col_area.width)
};
```

### 2.2 핵심 설계 결정

**(1) 줄 단위 판정**
- 조건은 paragraph 단위가 아니라 **줄(comp_line) 단위**. 같은 paragraph 안에서도 인라인 표가 있는 줄(line[0])과 없는 줄(line[2])이 다를 수 있다.
- pi=61 의 line[0]/[1] 는 sw=18939 (좁음), line[2] 는 sw=30562 (full). line[2] 는 새 분기 미진입 (`comp_line.segment_width < col_area_w_hu - 200` 불충족 — 30562 ≥ 31692-200 = 31492? 실제 30562 < 31492 라 조건 충족하긴 함).
- 단 line[2] 는 인라인 TAC 표가 없으므로 `line_has_inline_tac_table=false` → 새 분기 미진입.

**(2) 임계값 200 HU 재사용**
- 기존 `has_picture_shape_square_wrap` 분기와 동일한 임계값. 다단/회귀 차단 의도 일관.

**(3) 표만(Table tac=true) — 수식/Picture 제외**
- 인라인 수식(Equation tac=true) 은 작아서 sw 가 좁혀지지 않음(Stage 1 §5.3 검증).
- TAC Picture 도 동일.
- 표만 좁은 sw 를 유발 → Table 만 검출.

**(4) `has_picture_shape_square_wrap` 와 OR 결합**
- 기존 분기 보존 (Picture/Shape Square wrap 케이스 동일 출력).
- 두 조건 OR — 한쪽이라도 활성이면 narrow sw 적용.

### 2.3 이차 영향 — extra_word_sp 자동 정상화

`available_width` 가 narrow sw 기반으로 좁아지면:
- `effective_col_w = 252.5 px` (vs 기존 407.5)
- `available_width = 252.5 - 15.07 - 0 = 237.4 px`
- est_x: 미변경 (text_width + tac_width 계산은 effective_col_w 와 무관)
- `total_text_width ≈ 236.6 px` (변동 없음)
- `slack = 237.4 - 231.6 = 5.8 px` → `extra_word_sp = 5.8/2 = 2.9 px / space` ← **정상 범위**
- 표 시작 x = 564.94 + 2 × (5 + 2.9) = 580.7 px ≈ 기대값 ✓

`x_start` 도 자동 정상화 (effective_col_x = col_area.x + cs_px ≈ 549.87 + 15.07 = 564.94). 단 cs_px 가 line[0].column_start 이며, pi=61 ls[0].cs=1130 HU → 15.07 px → effective_col_x = 549.87 + 15.07 = 564.94. 그리고 effective_margin_left 는 별도로 더해지므로 **이중 적용** 위험.

**검증 필요**: cs_px 와 margin_left 가 의미상 중복인지, 별개 좌표축인지.

기존 `has_picture_shape_square_wrap` 분기에서는 `effective_col_x = col_area.x + cs_px` 후 `x_start = effective_col_x + effective_margin_left + ...` 로 모두 더한다. Picture wrap 케이스에서 이 분기로 정상 출력이 나왔다면, **cs_px = 한컴이 인코딩한 line 시작 절대 오프셋, margin_left = paragraph 단락 여백** 으로 별개 의미. 두 값은 일반적으로 동시에 0 이거나 한쪽만 의미를 가진다.

pi=61 의 ls[0].cs=1130 HU → 한컴이 인코딩한 line 시작 오프셋이 이미 15 px 들어가 있음. paragraph margin_left=2260 HU → resolver 가 /2 적용 후 15.07 px. **수치적으로 같은 15 px** — 한컴이 paragraph margin 을 LINE_SEG.cs 에도 반영해서 인코딩한 가능성 높음.

만약 cs + margin 이중 적용되면 표 x = 549.87 + 15.07(cs) + 15.07(margin) + 2×(5+2.9) ≈ 596 px → 16 px 추가 우측 편위 → 여전히 결함.

#### 2.3.1 분기 후보 — 이중 적용 회피

옵션 A1: 새 분기 진입 시 `effective_margin_left = 0` (cs 가 margin 역할 흡수했다고 가정)
옵션 A2: 새 분기 진입 시 `effective_col_x = col_area.x` (cs 무시), `effective_col_w = sw_px`
옵션 A3: 그대로 두고 (Picture wrap 분기와 동일) Stage 3 실측 후 보정

**Stage 3 에서 실측 결정**. 단, Picture wrap 분기에서 회귀 없이 동작하므로 이중 적용은 일반적으로 발생하지 않을 것 (한컴 인코딩 관습상 두 값이 보완적). 표 wrap 케이스만 다를 가능성 있음 → Stage 3 첫 build 후 SVG 좌표 실측으로 결정.

## 3. 변경 파일/LOC 추정

| 파일 | 변경 | 추정 LOC |
|------|------|---------|
| `src/renderer/layout/paragraph_layout.rs` | L857 직전 `line_has_inline_tac_table` 산출 + L857 조건 확장 | +12 / -1 |

기타 파일 변경 없음 (단일 분기 확장).

## 4. 회귀 검증 계획 (Stage 3)

### 4.1 단위 테스트 / 통합 테스트

```bash
cargo test --lib              # 1125+ 통과 확인
cargo test --test issue_546   # exam_science 페이지 수 4 유지
cargo clippy --all-targets -- -D warnings  # 신규 경고 0
```

### 4.2 svg_snapshot 골든 (필수)

```bash
cargo test --test svg_snapshot     # 6/6 통과
```

기존 골든 fixture 6 개 — exam_science 미포함 추정. byte-identical 보장.

### 4.3 광범위 fixture sweep

전체 `samples/*.hwp{,x}` (159 파일) 중 다음 기준 후보 선정:
1. **인라인 TAC 표 보유**: ck-table-fcn-table.hwp, atop-equation-01.hwp, equation-lim.hwp, exam_science.hwp, exam_math.hwp, exam_*.hwp, fraction*.hwp 등.
2. **인라인 수식 보유**: 같은 위 + 추가 candidates.
3. **다단 paragraph**: exam_*.hwp 모두 다단.
4. **Picture/Shape wrap=Square**: 21_언어_기출_편집가능본.hwp 등 — Stage 1 §4 의 narrow sw 보유 paragraph 의 정상 출력 보존 검증.

스크립트 (Stage 3 에서):
```bash
for f in samples/*.hwp samples/*.hwpx; do
  # 변경 전 SVG → /tmp/before/$(basename $f).svg
  # 변경 후 SVG → /tmp/after/$(basename $f).svg
  diff -q /tmp/before/...  /tmp/after/...
done
```

기준:
- exam_science.hwp: page 2/3/4 의도된 diff (12/13/15/16/19번 분수 위치 정정)
- 나머지: byte-identical

### 4.4 시각 판정 (Stage 4)

- 작업지시자 SVG 시각 판정 (12/13/15/16/19번 본문 정렬 + 7번 등 비회귀)
- 한컴 2010/2020 정답지 (PDF 200dpi) 비교 (보조 ref)

## 5. 위험 요소 및 완화

| 위험 | 발현 조건 | 완화책 |
|------|----------|-------|
| **cs/margin 이중 적용** | LINE_SEG.cs 가 paragraph margin 을 이미 흡수한 인코딩 | Stage 3 첫 빌드 후 SVG 좌표 실측 → 옵션 A1/A2/A3 선택 |
| Picture Square wrap 회귀 | 새 조건 `\|\|` 결합으로 기존 case 무영향 (조건 만족 시 동일 분기 진입) | 기존 fixture (21_언어_기출 등) byte-identical 검증 |
| 인라인 표 + full sw paragraph 회귀 | pi=79/110/118/120 — `comp_line.segment_width >= col_area_w_hu - 200` → 새 분기 미진입 | Stage 3 sweep 으로 byte-identical 확인 |
| 셀 내부 paragraph 효과 | cell_ctx + narrow sw + 인라인 표 — 의도된 정정 영향 | 광범위 sweep 에서 의도된 diff 만 발생 확인 |
| **메모리 `feedback_essential_fix_regression_risk` 정합** | 본질 정정은 회귀 위험 큼 | 광범위 sweep + 한컴 2010/2020 보조 검증 + 시각 판정 |

## 6. Stage 3 실행 순서

1. 코드 변경 (L857 분기 확장 + line_has_inline_tac_table 산출).
2. `cargo build --release` — 빌드 통과.
3. `target/release/rhwp export-svg samples/exam_science.hwp -o /tmp/svg_after/` — 본 사례 SVG 생성.
4. pi=61 표 좌표 (debug-overlay) 측정 — 정상 (~580 px) 확인.
5. 이중 적용 발견 시 옵션 A 분기 선택, 재빌드.
6. `cargo test --lib`, `cargo clippy`, `cargo test --test svg_snapshot` — 통과.
7. 광범위 sweep — byte-identical 확인 (의도된 diff 만 exam_science).
8. Stage 3 보고서 작성.

## 7. 산출물 (Stage 3)

- `mydocs/working/task_m100_568_stage3.md` — 구현 + 검증 결과
- 코드 diff: `src/renderer/layout/paragraph_layout.rs`
- (필요 시) sweep 결과 요약

## 8. 승인 요청

본 구현 계획대로 Stage 3 (구현 + 검증) 진입을 승인 요청합니다.
