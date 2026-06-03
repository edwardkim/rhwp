# 단계2 완료 보고서: is_narrow_punctuation 헬퍼 + base_w 분기

- **타스크**: [#257](https://github.com/edwardkim/rhwp/issues/257)
- **마일스톤**: M100
- **브랜치**: `local/task257`
- **작성일**: 2026-04-23
- **단계**: 2 / 4

## 1. 목표

메트릭 DB 미등록 폰트의 폴백 경로에서 narrow glyph(콤마·중점 등) advance 를 `font_size * 0.5` → `font_size * 0.3` 으로 수정하여, `text-align-2.hwp` 표 셀의 `1,000` · `어휘·표현` 이 PDF 출력과 수렴하도록 한다.

## 2. 수행 내용

### 2.1 `is_narrow_punctuation` 헬퍼 추가

`src/renderer/layout/text_measurement.rs:866-873`:

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

화이트리스트 8자: `,` `.` `:` `;` `'` `"` `` ` `` `·`.

### 2.2 폴백 경로 3곳에 분기 추가

| 위치 | 함수 | 수정 라인 |
|------|-----|----------|
| `text_measurement.rs:184-193` | `EmbeddedTextMeasurer::estimate_text_width` | 폴백에 narrow 분기 |
| `text_measurement.rs:286-295` | `EmbeddedTextMeasurer::compute_char_positions` | 폴백에 narrow 분기 |
| `text_measurement.rs:809-817` | `estimate_text_width_unrounded` (free fn) | 폴백에 narrow 분기 |

수정 형태 (동일):

```rust
let base_w = if let Some(w) = measure_char_width_embedded(...) {
    w
} else if cluster_len[i] > 1 || is_cjk_char(c) || is_fullwidth_symbol(c) {
    font_size
} else if is_narrow_punctuation(c) {
    // Task #257: 콤마·중점 등 narrow glyph 폴백 폭 (0.5 → 0.3).
    font_size * 0.3
} else {
    font_size * 0.5
};
```

min_w 클램프(Task #229)는 그대로 유지. base_w 가 작아짐에 따라 클램프 하한도 자동으로 `0.3 * ratio * 0.5 = 0.15 × font_size` 로 축소 — 안전성은 유지되고 하한이 실제 글리프 폭에 더 가까워짐.

### 2.3 `#[ignore]` 해제 + 테스트 결과

단계 1 에서 `#[ignore]` 로 격리했던 4건 활성화. 단독 실행 결과:

```
test_narrow_glyph_comma_base_width ... ok
test_narrow_glyph_middle_dot_base_width ... ok
test_narrow_glyph_period_and_colon ... ok
test_non_narrow_char_unchanged ... ok
```

전체 `text_measurement::` 모듈:

```
running 22 tests
......................
test result: ok. 22 passed; 0 failed; 0 ignored
```

Task #229 회귀 테스트 4건 포함 **전부 pass** (단조성 보장 유지).

### 2.4 `text-align-2.hwp` 재생성 수치 비교

표 셀 HY중고딕 font-size=16.667 (폴백 경로):

**어휘·표현**:
| 문자 시퀀스 | BEFORE advance | AFTER advance | 변화 |
|-------------|---------------|---------------|------|
| 어→휘 (CJK) | 16.00 | 16.00 | 변화 없음 |
| 휘→· (CJK) | 16.00 | 16.00 | 변화 없음 |
| **·→표 (narrow)** | **7.67** | **4.33** | -3.34 px (-43%) |
| 표→현 (CJK) | 16.00 | 16.00 | 변화 없음 |

**1,000**:
| 문자 시퀀스 | BEFORE advance | AFTER advance | 변화 |
|-------------|---------------|---------------|------|
| 1→, (digit) | 7.67 | 7.67 | 변화 없음 |
| **,→0 (narrow)** | **7.67** | **4.33** | -3.34 px (-43%) |
| 0→0 | 7.67 | 7.67 | 변화 없음 |
| 0→0 | 7.67 | 7.67 | 변화 없음 |

narrow advance 수식 검증:
- AFTER = `font_size * 0.3 + letter_spacing = 16.667 * 0.3 + (-0.67) ≈ 5.00 - 0.67 = 4.33` ✓
- 기대 계수 = `0.26 × font_size` 실측, 목표 `≤ 0.35` 달성

### 2.5 종합 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib text_measurement::` | **22 pass / 0 fail** |
| `cargo test --lib renderer::` | **285 pass / 0 fail** (전체 renderer 회귀 방어) |
| `cargo test --test svg_snapshot` | **3 pass / 0 fail** (스냅샷 골든 변경 없음) |
| `cargo clippy --lib -- -D warnings` | **통과** (경고 없음) |

### 2.6 사전 존재 실패 (본 타스크 무관)

`cargo test --lib` 전체 실행 시 14건 fail 존재:
- `serializer::cfb_writer::tests::*` (2건)
- `wasm_api::tests::*` (12건)

**본 수정 전 커밋(b7e62bd)에서도 동일 실패 재현 확인** — `git stash` 후 `test_save_text_only` 실행 결과 동일하게 fail. 본 타스크와 무관한 사전 이슈 (CFB 라운드트립 · WASM fixture 관련).

## 3. svg_snapshot 영향

`cargo test --test svg_snapshot` 결과 **3 건 모두 pass** — 골든 재생성 **불필요**.

이유: 현재 `svg_snapshot` 테스트는 narrow glyph 미등록 폰트 (HY 계열) 를 쓰지 않거나, 미포함된 좌표만 비교하는 것으로 보임. `biz_plan.hwp` 같은 스모크 테스트는 단계 4 에서 별도 수동 점검.

## 4. 다음 단계 (단계 3) 예고

단계 2 결과 narrow advance 가 이미 `4.33 px` (≈ `0.26 × font_size`) 로 수렴. 이제 단계 3 에서:

1. **PDF 환산값과 잔차 측정**: mutool 로 PDF 150dpi PNG 추출 → 동일 좌표 지점의 실제 `·`·`,` 위치 계측
2. **잔차 ≤ 1 px 판정**:
   - 만족 시 → `min_w` 클램프 추가 손댐 **불필요**. 단계 보고서에 "검증 완료, 수정 없음" 기록
   - 초과 시 → `is_narrow_punctuation` 시 clamp 우회 분기 추가 + Task #229 monotonic 테스트 재검증
3. **등록 경로(`measure_char_width_embedded`) 추가 수정 여부**: 휴먼명조 본문 `세대별·지역별` 등에서 `·` advance 관찰 (단계 1 §5 관찰 사항 추적)

## 5. 산출물

- `src/renderer/layout/text_measurement.rs`:
  - `is_narrow_punctuation` 헬퍼 신규
  - 폴백 경로 3곳 (`estimate_text_width`, `compute_char_positions`, `estimate_text_width_unrounded`) 에 narrow 분기
  - 기존 `#[ignore]` 테스트 4건 활성화
- `output/svg/text-align-2/text-align-2.svg` (수정 후, narrow glyph advance 4.33 px)
- `mydocs/working/task_m100_257_stage2.md` (본 보고서)

## 6. 리스크·관찰

| 관찰/리스크 | 대응 |
|-------------|------|
| 휴먼명조 본문 `,` advance 14.56 px (0.73 × font_size, 등록 경로) | 등록 경로는 폰트 메트릭 기반이므로 본 수정 대상 아님. 단계 3 에서 PDF 와 비교하여 추가 조치 필요 여부 판단 |
| `text-align-2.hwp` 전체 배치 좌측 shift 발생 (사소) | 표 셀 content 앞에 narrow glyph(`·`) 있어 누적 shift. PDF 와 비교 후 허용 범위인지 확인 |
| 단계 1 관찰한 `measure_char_width_embedded` 내 U+2018-2027/U+00B7 강제 em/2 | 등록 폰트 경로. 본 단계 수정 범위 아님 |

## 7. 요청 사항

단계 2 는 계획대로 완료했습니다. Task #229 회귀 테스트 포함 모든 renderer 테스트(285건) 통과.

승인 시 단계 3 (clamp 검증 · 등록 경로 영향 분석) 진행.
