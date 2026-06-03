# 최종 결과보고서: narrow glyph(콤마·중점 등) 뒤 문자 advance 과다 보정

- **타스크**: [#257](https://github.com/edwardkim/rhwp/issues/257)
- **마일스톤**: M100
- **브랜치**: `local/task257` (from `local/devel`)
- **작성일**: 2026-04-23

## 1. 요약

Task #146 종결 직후 회귀 검증 목적으로 작업지시자가 제공한 `text-align-2.hwp` 에서 2개 버그 발견:

1. **폴백 경로 narrow glyph advance 과다**: 메트릭 DB 미등록 폰트(HY중고딕 등)에서 콤마·중점·마침표 등 narrow glyph 가 반각(`font_size × 0.5`) 폭으로 계산되어 PDF 대비 뒷 글자가 2~3 px 우측 밀림.
2. **`·` 중점 폰트 대체 시각 불균형**: `.hwp` 지정 폰트(휴먼명조) 미설치 환경에서 Batang 등으로 대체될 때 `·` 글리프의 LSB·폭이 달라 시각적으로 한쪽 이웃에 쏠림.

두 버그를 각기 **metric 경로 수정(단계 2)** 과 **SVG 렌더러 우회(단계 3)** 로 해결.

## 2. 버그·원인·수정 매트릭스

| 번호 | 증상 | 원인 | 수정 | 단계 |
|------|------|------|------|------|
| B-1 | 표 셀 `1,000항목` → `1, 000항목` | `compute_char_positions` 폴백에서 `base_w = font_size × 0.5` | `is_narrow_punctuation` 헬퍼 + `font_size × 0.3` 분기 (3곳) | 2 |
| B-1 | 표 헤더 `어휘·표현` → `어휘· 표현` | 동 | 동 | 2 |
| B-2 | 본문 `세대별·지역별` · 사이 `·` 이 오른쪽(지)쪽으로 쏠림 | Batang 의 `·` 글리프가 휴먼명조와 다른 LSB 로 렌더됨 (폰트 대체 문제) | SVG 에서 `·` 를 `<text>` 대신 `<circle>` 로 직접 그림 (폰트 독립) | 3 |

## 3. 좌표·시각 수렴 결과

### 3.1 표 셀 HY중고딕 16.67pt `어휘·표현`

| 문자 시퀀스 | 수정 전 advance | 수정 후 advance | 기대 효과 |
|-------------|---------------|---------------|----------|
| 휘 → · | 16.00 px (CJK) | 16.00 px | 변경 없음 |
| · → 표 | **7.67 px (= 0.460 × font_size, 반각)** | **4.33 px (= 0.260 × font_size)** | 콤마 뒤 공백 해소 |

narrow base_w 변화: `0.5 × font_size` → `0.3 × font_size`.
실 advance: `0.3 × font_size - letter_spacing ≈ 0.26 × font_size` (letter_spacing 반영).

### 3.2 표 셀 HY중고딕 16.67pt `1,000항목`

| 시퀀스 | 수정 전 | 수정 후 |
|--------|--------|--------|
| 1 → , | 7.67 | 7.67 (변화 없음) |
| **, → 0** | **7.67** | **4.33** |
| 0 → 0 | 7.67 | 7.67 |

### 3.3 본문 휴먼명조 20pt `별·지`

| 지표 | 수정 전 (A안 초안/refinement) | 최종 (C안) |
|------|---------------------------|-----------|
| · 렌더 수단 | `<text>` + shift 보정 | **`<circle>` 직접 그리기** |
| · 위치 | Batang LSB 때문에 오른쪽 쏠림 | advance 박스 수평 **중앙** 고정 |
| 폰트 대체 영향 | 브라우저/OS 마다 다름 | **없음** (폰트 독립) |

SVG 실례:
```xml
<!-- C안: 폰트 독립 circle 렌더 -->
<circle cx="174.9422" cy="151.4133" r="1.6000" fill="#000000"/>
```

## 4. 검증

### 4.1 자동 테스트

| 테스트 | 결과 |
|--------|------|
| `cargo test --lib` | 937 passed / 14 failed (14건은 본 PR 전부터 실패, `serializer::cfb_writer` · `wasm_api`) |
| `cargo test --lib text_measurement::` | **22 pass / 0 fail** (신규 테스트 4건 + Task #229 회귀 4건 포함) |
| `cargo test --lib renderer::` | **285 pass / 0 fail** |
| `cargo test --test svg_snapshot` | **3 pass** (form-002 golden 1회 재생성: `·` `<text>` → `<circle>`) |
| `cargo clippy --lib -- -D warnings` | **clean** |

### 4.2 시각 검증 (Chrome 150dpi · 4x 확대)

| 샘플 | 관찰 |
|------|------|
| `samples/text-align-2.hwp` | `·` / `,` 모두 PDF 와 시각적 근사. "별·지", "시·청" 등 `·` 중앙 배치 |
| `samples/biz_plan.hwp` | 591개 `·` 모두 `<circle>` 로 균일 렌더. TOC 리더 도트 일관성 확보 |
| `samples/exam_kor`, `exam_eng`, `exam_math`, `footnote-01`, `field-01` | 회귀 없음. 각 문서 내 `,` `.` `·` 정상 |

### 4.3 사전 존재 실패 (본 PR 무관)

14건 fail:
- `serializer::cfb_writer::tests::*` (2건) — CFB 라운드트립 fixture 이슈
- `wasm_api::tests::*` (12건) — HWP 라운드트립 · 셀 merge · 텍스트 삽입 관련

본 타스크 첫 커밋 이전 (`git stash` 후 baseline) 에서도 동일 실패 재현 확인. Task #257 수정과 무관.

## 5. 구현 세부

### 5.1 단계 2 — `is_narrow_punctuation` 헬퍼 + 폴백 `base_w` 분기

`src/renderer/layout/text_measurement.rs`:

```rust
/// 실제 글리프 폭이 반각(em/2)보다 뚜렷이 좁은 구두점·기호.
/// 메트릭 DB 미등록 폰트의 폴백 폭 계산 시 `font_size * 0.5` 대신
/// `font_size * 0.3` 을 쓰도록 분기하는 기준 (Task #257).
fn is_narrow_punctuation(c: char) -> bool {
    matches!(c,
        ',' | '.' | ':' | ';' | '\'' | '"' | '`' |
        '\u{00B7}'   // · MIDDLE DOT
    )
}
```

분기 적용 3곳:
- `EmbeddedTextMeasurer::estimate_text_width` (line 184)
- `EmbeddedTextMeasurer::compute_char_positions` (line 286)
- `estimate_text_width_unrounded` (free fn, line 809)

Task #229 의 음수 자간 `min_w` 클램프(단조성 보장)는 그대로 유지. narrow base_w 로 하한도 자동 축소 (`0.15 × font_size`).

### 5.2 단계 3 — `·` 를 SVG `<circle>` 로 렌더

`src/renderer/svg.rs` `draw_text`:

```rust
let is_middle_dot = |cluster_str: &str| cluster_str == "\u{00B7}";
let dot_radius = font_size * 0.08;
let dot_cy_offset = -font_size * 0.35;

// ... 렌더 루프 내부:
if is_middle_dot(cluster_str) {
    let adv = cluster_advance(*char_idx, cluster_str);
    let cx = x + char_positions[*char_idx] + adv / 2.0;
    let cy = y + dot_cy_offset;
    self.output.push_str(&format!(
        "<circle cx=\"{:.4}\" cy=\"{:.4}\" r=\"{:.4}\" fill=\"{}\"/>\n",
        cx, cy, dot_radius, color,
    ));
    continue;
}
```

**설계 원칙**:
- advance 계산은 그대로 (`measure_char_width_embedded` 변경 없음, `·` = em/2)
- 오직 렌더링 단계에서만 `·` 를 벡터 도형으로 대체
- `cx = advance 박스 수평 중앙`, `cy = baseline - 0.35 × font_size` (CJK x-height 중앙 근사), `r = 0.08 × font_size`
- 그림자 렌더링 루프에도 동일 분기 (shadow color 적용)

### 5.3 신규 단위 테스트 4건

`text_measurement.rs:tests`:

1. `test_narrow_glyph_comma_base_width` — HY헤드라인M, 13.333pt, `"A,B"` → 콤마 advance ≤ `font_size × 0.35`
2. `test_narrow_glyph_middle_dot_base_width` — 16.667pt, `"가·나"`
3. `test_narrow_glyph_period_and_colon` — `"A.B"`, `"A:B"`
4. `test_non_narrow_char_unchanged` — `A` 와 `가` 가 narrow 분기에 걸리지 않는지 회귀 방어

Task #229 의 기존 회귀 테스트 4건(`test_overflow_compression_positions_monotonic_*`, `test_charshape_negative_letter_spacing_no_reverse`, `test_non_compression_width_unchanged_by_fix`) 모두 pass 유지.

## 6. 범위 밖 (후속 고려)

| 항목 | 상태 |
|------|------|
| 한컴 proprietary 폰트 전반 임베딩 | 별도 마일스톤 (M101~). `·` 외 글리프도 폰트 대체 영향 있음 |
| 오픈소스 대체 폰트 번들링 (Noto Serif KR 등) | `mydocs/tech/font_fallback_strategy.md` 범위, 별도 이슈 |
| `,` `.` `:` 등 baseline 구두점 렌더 | 현재는 기존 `<text>` 유지. 폰트 대체 영향 상대적으로 작음. 필요 시 별도 이슈 |
| `min_w` 클램프 narrow glyph 우회 | 검증 결과 **불필요**. 현재 공식 그대로 Task #229 회귀 방어 유지 |
| `·` `cy_offset = -0.35 × font_size` 파라미터 | text-align-2 기준 튜닝. 다른 문서에서 미세 조정 필요 시 후속 이슈 |

## 7. 커밋 이력

```
85cd577 Task #257 단계3 최종: · 중점을 폰트 독립 <circle> 로 렌더 (C안)
5f6496b Task #257 단계3 refinement: · 중앙 배치 공식에 prev trailing bearing 반영
e47ca90 Task #257 단계3: · 중점 시각적 중앙 배치 (A안)
c76f2d3 Task #257 단계2: is_narrow_punctuation 헬퍼 + base_w 분기 (콤마/중점 등)
b7e62bd Task #257 단계1: narrow glyph baseline 측정 + 단계별 보고서
c556df3 Task #257: 샘플 samples/text-align-2.hwp / samples/text-align-2.pdf 편입
```

(A안·refinement 커밋은 설계 의사결정 히스토리로 보존 — 머지 전 squash 여부는 작업지시자 판단)

## 8. 파일 변경 요약

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/text_measurement.rs` | `is_narrow_punctuation` 헬퍼 + 폴백 분기 3곳 + 단위 테스트 4건 |
| `src/renderer/svg.rs` | `draw_text` 에 `·` → `<circle>` 렌더 분기 (그림자·본문 양쪽) |
| `tests/golden_svg/form-002/page-0.svg` | `·` `<text>` → `<circle>` 재생성 |
| `samples/text-align-2.hwp`, `.pdf` | 신규 편입 |
| `mydocs/plans/task_m100_257.md`, `_impl.md` | 계획서 |
| `mydocs/working/task_m100_257_stage{1,2,3,4}.md` | 단계별 보고서 |
| `mydocs/report/task_m100_257_report.md` | 본 최종 보고서 |
| `mydocs/tech/text_align_2_svg_pdf_compare.md` | 사전 비교 조사 (이전 작성) |
| `mydocs/orders/20260423.md` | Task #257 진행 상태 갱신 |

## 9. 참조

- 사전 비교: `mydocs/tech/text_align_2_svg_pdf_compare.md`
- 비교 이미지: `output/compare/text-align-2/pdf-1.png`, `svg-chrome150.png`
- 최종 SVG: `output/svg/text-align-2/text-align-2.svg`
- baseline/중간/최종 SVG 스냅샷: `output/re/text-align-2-{baseline,stage3,stage3-final,stage3-circle}-task257.svg`
- 선행 타스크: [#146](https://github.com/edwardkim/rhwp/issues/146) (Geometric Shapes · TAC · heavy face bold)
- 안전망 근거: Task #229 (narrow glyph 단조성 회귀 테스트 4건 — 본 PR 에서도 계속 pass)

## 10. 머지·클로즈 승인 대기

`local/task257` → `local/devel` → `devel` 머지 및 `#257` 이슈 클로즈는 작업지시자 승인 후 수행.
