# 단계1 완료 보고서: narrow glyph baseline 측정

- **타스크**: [#257](https://github.com/edwardkim/rhwp/issues/257)
- **마일스톤**: M100
- **브랜치**: `local/task257`
- **작성일**: 2026-04-23
- **단계**: 1 / 4

## 1. 목표

수정 전 수치를 테스트·SVG·측정값으로 고정하여 단계 2~4 개선 여부를 정량 판정할 baseline 확보.

## 2. 수행 내용

### 2.1 샘플 파일 편입

- `text-align-2.hwp` · `text-align-2.pdf` 를 루트 → `samples/` 로 이동
- `git check-ignore samples/text-align-2.{hwp,pdf}` → 양쪽 모두 exit=1 (gitignored 아님, 정상 추적 가능)

### 2.2 Baseline SVG 생성

```bash
cargo run --bin rhwp -- export-svg samples/text-align-2.hwp -o output/svg/text-align-2/
```

- 출력: `output/svg/text-align-2/text-align-2.svg` (109,738 bytes)
- 참조 보관: `output/re/text-align-2-baseline-task257.svg` (동일 내용, 단계 2~4 비교용)

### 2.3 narrow glyph advance 현재 수치 (SVG 좌표 기반)

표 셀 (HY중고딕, font-size=16.667, 메트릭 DB 미등록 → 폴백):

**어휘·표현** (line 94~98, y=262.28):
| 문자 | x 좌표 | 직전 대비 advance |
|------|--------|------------------|
| 어 | 138.00 | — |
| 휘 | 154.00 | 16.00 (= font_size, CJK 전각) |
| · | 170.00 | 16.00 (= font_size, 전각 처리) |
| 표 | 177.67 | **7.67 (= 0.460 × font_size)** |
| 현 | 193.67 | 16.00 |

**1,000** (line 104~108, y=243.11):
| 문자 | x 좌표 | 직전 대비 advance |
|------|--------|------------------|
| 1 | 301.41 | — |
| , | 309.08 | 7.67 (= 0.460 × font_size) |
| 0 | 316.75 | **7.67 (= 0.460 × font_size)** |
| 0 | 324.41 | 7.67 |
| 0 | 332.08 | 7.67 |

**관찰**: `·` advance 7.67 px, `,` advance 7.67 px — **반각 advance(= `font_size * 0.5 - letter_spacing`)** 와 일치. 숫자 `1`·`0` 과 동일 폭을 부여받고 있음 → 증상 재현.

letter_spacing 역산: `7.67 = 0.5 × 16.667 - x` → `x ≈ 0.66` px 음수 자간 (≈ -4% of font_size, CharShape 기본값 근방)

### 2.4 기존 테스트 baseline 확인

```bash
cargo test --lib text_measurement::
# running 18 tests ... test result: ok. 18 passed; 0 failed
```

Task #229 회귀 테스트 4건 전부 pass:
- `test_overflow_compression_positions_monotonic_comma`
- `test_charshape_negative_letter_spacing_no_reverse`
- `test_overflow_compression_positions_monotonic_period`
- `test_non_compression_width_unchanged_by_fix`

### 2.5 단계 2 용 failing 테스트 4건 추가 (`#[ignore]`)

`src/renderer/layout/text_measurement.rs:1253~1350` 근방에 신규:

| 테스트 | 조건 | 기대 (단계 2 후) |
|--------|------|----------------|
| `test_narrow_glyph_comma_base_width` | HY헤드라인M, 13.333pt, `"A,B"` | comma advance ≤ `font_size * 0.35` |
| `test_narrow_glyph_middle_dot_base_width` | HY헤드라인M, 16.667pt, `"가·나"` | · advance ≤ `font_size * 0.35` |
| `test_narrow_glyph_period_and_colon` | HY헤드라인M, 13.333pt, `"A.B"`·`"A:B"` | `.`·`:` advance ≤ `font_size * 0.35` |
| `test_non_narrow_char_unchanged` | HY헤드라인M, 13.333pt, `"AA"`·`"가가"` | `A` advance = 반각, `가` = 전각 (회귀 방어) |

**baseline 검증**:

```bash
$ cargo test --lib test_narrow_glyph -- --ignored
... test result: FAILED. 0 passed; 3 failed
narrow comma advance should be ≤ font_size * 0.35 (4.67), got 6.67
narrow middle-dot advance should be ≤ font_size * 0.35 (5.83), got 8.33
narrow '.' advance should be ≤ font_size * 0.35 (4.67), got 6.67

$ cargo test --lib test_non_narrow_char_unchanged -- --ignored
... test result: ok. 1 passed
```

→ 3건은 현재 **실패** (버그 재현), 1건은 현재 **통과** (회귀 방어 기준점 확립) — 의도대로 세팅됨.

## 3. 다음 단계 (단계 2) 예고

- `is_narrow_punctuation(c)` 헬퍼 추가 (`text_measurement.rs:~859` is_fullwidth_symbol 뒤)
- `compute_char_positions` 폴백 3곳 (`line 184`, `278`, `793`) 에 narrow 분기 추가: `font_size * 0.3`
- 신규 테스트 4건 `#[ignore]` 제거 → 전부 pass
- Task #229 기존 테스트 4건 계속 pass

## 4. 산출물

- `samples/text-align-2.hwp`, `samples/text-align-2.pdf` (편입)
- `output/svg/text-align-2/text-align-2.svg` (baseline)
- `output/re/text-align-2-baseline-task257.svg` (보관용 복사본)
- `src/renderer/layout/text_measurement.rs` (`#[ignore]` 테스트 4건 추가)
- `mydocs/working/task_m100_257_stage1.md` (본 보고서)

## 5. 리스크·관찰

### 관찰: `measure_char_width_embedded` 의 이중 경로

조사 중 발견: `measure_char_width_embedded` (`text_measurement.rs:731-746`) 는 메트릭 DB 에 폰트가 등록된 경우, `·`(U+00B7)·스마트 따옴표(U+2018..U+2027) 를 **`em/2` 로 강제** 하는 로직 존재.

```rust
let is_halfwidth_punct = matches!(c,
    '\u{2018}'..='\u{2027}' | '\u{00B7}'
);
if is_halfwidth_punct && glyph_w >= mm.metric.em_size {
    mm.metric.em_size / 2  // 반각 강제
}
```

→ **메트릭 DB 등록 폰트**(예: 휴먼명조)도 `·` 에 `font_size * 0.5` advance 부여 → **동일 bug 가 등록 폰트에서도 발생 가능**.

`text-align-2.hwp` 의 본문 `세대별·지역별` 는 휴먼명조(등록) 이므로 이 경로를 탐. SVG line 35 `·` x=170.74, line 40 `어` x=263.52 (중간 여러 글자) — 별도 검증 필요.

**단계 2 방침**:
- 폴백 경로 수정(우선) → 표 셀 HY중고딕·본문 HY헤드라인M 증상 해결
- 등록 경로(`measure_char_width_embedded` line 737-746) 는 **손대지 않음** (Task #229 등 누적 회귀 위험). 필요 시 후속 이슈 또는 단계 3 에서 PDF 수치 비교 후 판단

### 리스크

| 리스크 | 대응 |
|--------|-----|
| 등록 경로 수정 범위 확장 요구 | 단계 3 에서 수치 확인. 폴백 수정만으로 목표(≤ 1 px 잔차) 달성 시 등록 경로는 건드리지 않음 |
| `font_size * 0.3` 계수가 과도하게 좁을 경우 `1`·`0` 간 간격 자체가 영향 받지 않음 (narrow 분기 따로) — 확인됨 | 영문·숫자는 narrow 아님. `test_non_narrow_char_unchanged` 로 방어 |

## 6. 요청 사항

단계 1 은 **코드 수정 없음** 원칙에서 일탈 없이 수행 완료 (테스트 `#[ignore]` 로 격리). 승인 시 단계 2 진행.
