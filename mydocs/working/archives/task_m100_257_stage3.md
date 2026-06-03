# 단계3 완료 보고서: · 중점 시각적 중앙 배치 (C안 — `<circle>` 렌더)

- **타스크**: [#257](https://github.com/edwardkim/rhwp/issues/257)
- **마일스톤**: M100
- **브랜치**: `local/task257`
- **작성일**: 2026-04-23
- **단계**: 3 / 4

## 1. 배경

단계 2 (폴백 경로 narrow glyph base_w 분기) 완료 후 `·` 위치 개선 요청 → 수차례 시도 끝에 **폰트 대체가 근본 원인** 으로 확인. 최종 C안(폰트 비의존 `<circle>` 렌더) 채택.

### 이전 시도 (모두 실패)

| 시도 | 방향 | 결과 |
|-----|------|------|
| A안 초기 | `measure_char_width_embedded` 에서 `·` advance 를 0.3 em 으로 축소 | 좌우 gap 불균형 (우측 쏠림 심화) |
| A안 refinement | SVG x 에 `(advance - glyph_w) / 2` shift 추가 | 여전히 쏠림 (이중 shift) |
| A안 refinement 2 | `(advance - prev_bearing - glyph_w) / 2` | 수치상 균형 개선 but 브라우저 렌더에서 `·` 이 지 쪽 밀착 |

### 근본 원인

`.hwp` 는 휴먼명조 지정 → 사용자 환경에 휴먼명조 없음 → Batang 으로 폰트 대체 → **Batang 의 `·` 글리프 LSB·폭 이 휴먼명조와 달라** rhwp 의 advance 기반 위치 계산과 불일치. shift 보정 시도는 font-dependent 하여 일관성 없음.

## 1-1. 단계 3 원 배경 (참고)

단계 2 완료 후 작업지시자 피드백으로 `text-align-2.svg` 의 등록 폰트 경로(휴먼명조) 본문 `·` 도 narrow 로 수렴해 달라는 요청이 있었다. 단계 3 진입점에서 `measure_char_width_embedded` 의 is_halfwidth_punct 목록에서 `·` 를 `is_narrow_punctuation` 으로 이관 + 0.3 em 상한 캡을 먼저 시도했다 (초안 구현).

그 결과 본문 `세대별·지역별` · `시·청각장애인의` 의 `·` advance 가 8.40 → 5.37 px 로 축소됐으나, 작업지시자 재검증에서 **`·` 글리프가 우측(지·청) 이웃에 쏠려 보인다** 는 지적.

측정 결과 `·` 글리프 좌 여백 3.40 px > 우 여백 1.37 px 로 실제 "오른쪽 쏠림" 확인. 원인은 advance 자체가 문제 아니라 **글리프가 advance box 왼쪽에 자연 정렬** 되어 중앙 배치되지 않는 것. 한컴 PDF 는 advance=em/2 + 글리프 중앙 배치 조합.

## 2. 최종 방침 (C안)

`·` 글리프를 **폰트와 완전 무관** 하게 SVG `<circle>` 로 직접 그린다. rhwp 의 advance 계산은 그대로 사용 (레이아웃 영향 없음), 단지 "그 advance 박스의 수평 중앙" 에 점을 찍는다.

| 항목 | C안 (최종 채택) |
|-----|----------------|
| `measure_char_width_embedded` | 기존 로직 (em/2 for `·`, 변경 없음) |
| `·` advance | em/2 (휴먼명조 20pt → 8.40 px) |
| SVG 렌더링 | `<circle cx=".." cy=".." r=".." fill=".."/>` |
| 폰트 의존성 | **없음** (모든 브라우저/OS 동일 렌더) |
| 스코프 | `·` (U+00B7) 1자만. `,` `.` 등은 기존 `<text>` 유지 |

## 3. 수행 내용

### 3.1 `measure_char_width_embedded` 는 변경 없음

단계 2 이전 상태 그대로 유지:

```rust
let is_halfwidth_punct = matches!(c,
    '\u{2018}'..='\u{2027}' | '\u{00B7}'
);
if is_halfwidth_punct && glyph_w >= mm.metric.em_size {
    mm.metric.em_size / 2
} else {
    glyph_w
}
```

- `·` advance 는 em/2 (HWP 관례) 그대로 → 다음 글자 위치 영향 없음
- 단계 2 의 `is_narrow_punctuation` 헬퍼는 **폴백 경로에서만 작동** (그대로 유지)

### 3.2 SVG 렌더러 `·` → `<circle>` 분기 (신규)

`src/renderer/svg.rs:1793~` `draw_text` 에 중앙 배치 원형 도형 분기 추가:

```rust
let is_middle_dot = |cluster_str: &str| cluster_str == "\u{00B7}";
let dot_radius = font_size * 0.08;
let dot_cy_offset = -font_size * 0.35;

for (char_idx, cluster_str) in &clusters {
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
    // ...일반 <text> 렌더
}
```

**설계 값**:
- `cx = advance 박스 수평 중앙`
- `cy = baseline(y) − font_size × 0.35` → CJK x-height 중앙 근사
- `r = font_size × 0.08` → PDF 관찰치 기준 도트 반지름
- `fill` = 텍스트 색상 동일

**적용 대상**: `·` (U+00B7) 1자만. `,` `.` `:` 등 baseline 글리프는 기존 `<text>` 유지.

**그림자·본문 렌더링 루프** 양쪽 동일 분기 적용.

### 3.3 수치 비교 (휴먼명조 20pt 본문 `별·지`)

C안은 `·` 를 **벡터 원형** 으로 그리므로 폰트 글리프의 LSB/폭 이슈가 완전히 사라진다.

- 별 x=152.34, 별 advance 18.40 → 별 advance 끝 = 170.74
- 지 x=179.14 (다음 글자 위치, 변경 없음)
- · circle cx = (170.74 + 179.14) / 2 = **174.94** (advance box 수평 중앙, SVG)
- · circle r = 20 × 0.08 = 1.60 px
- · circle cy = 158.41 − 20 × 0.35 = **151.41** (x-height 중앙)

브라우저 렌더 결과: `·` 이 별과 지 사이 정확히 가운데에 위치. 폰트 대체와 무관.

SVG 실례 (line 35):
```xml
<circle cx="174.9422" cy="151.4133" r="1.6000" fill="#000000"/>
```

### 3.4 HY중고딕 표 셀 (폴백 경로) 에서도 동일 원리

단계 2 의 narrow advance (0.3 em) 가 그대로 적용된 상태에서:
- advance = 4.33 px, cx = advance 중앙 = 170.74 + 2.165 → 시각상 좁은 공간 안에서도 정확 중앙
- `어휘·표현` 표 헤더 셀에서 `·` 이 휘·표 사이에 깔끔하게 위치

### 3.5 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib text_measurement::` | **22 pass / 0 fail** |
| `cargo test --lib renderer::` | **285 pass / 0 fail** |
| `cargo test --test svg_snapshot` (golden 재생성 후) | **3 pass** |
| `cargo clippy --lib -- -D warnings` | **통과** |

Task #229 회귀 테스트 4건도 baseline 그대로 통과 (advance 계산 로직은 단계 3 에서 변경 없음, `·` 에만 렌더링 shift 적용).

svg_snapshot golden 은 form-002·table-text 양쪽 재생성 (narrow `·` 렌더링 shift 로 인한 x 좌표 변경 반영).

## 4. 잔여 과제 (단계 4 로 이관)

- `min_w` 클램프 우회 검토: **불필요** 로 최종 확정. 사유:
  - 등록 경로 advance 는 원래 em/2 복귀 (narrow cap 없음)
  - 폴백 경로 narrow cap(0.3 em)은 단계 2 에서 그대로 유지, Task #229 회귀 테스트 전부 통과
- 스모크 스위프 (narrow glyph 다수 샘플): 단계 4
- 최종 결과보고서: 단계 4

## 5. 산출물

- `src/renderer/svg.rs`:
  - `draw_text` 에 `·` (U+00B7) → `<circle>` 렌더 분기 추가 (그림자·본문 양쪽)
  - 이전 시도(shift 기반 중앙 배치) 로직 제거
- `src/renderer/layout/text_measurement.rs`:
  - 변경 없음 (단계 2 상태 그대로)
- `tests/golden_svg/form-002/page-0.svg`: 재생성 (`·` `<text>` → `<circle>`)
- `output/svg/text-align-2/text-align-2.svg` (수정 후)
- `output/re/text-align-2-stage3-circle-task257.svg` (참조 보관)
- `mydocs/working/task_m100_257_stage3.md` (본 보고서)

## 6. 한계와 범위 밖 (후속 고려)

- **타 narrow glyph (`,` `.` `:` `;` 등) 는 기존 `<text>` 유지**: 이들은 baseline 에 자연 배치되므로 폰트 대체 영향이 상대적으로 작다. 필요 시 별도 이슈로 확장.
- **한컴 proprietary 폰트 전반의 글리프 품질 차이**: `·` 외에도 획 두께, letter-spacing 미세 차이 등이 폰트 대체 시 발생. 근본 해결은 **폰트 번들/임베딩 전략(M101~)** 범위.
- **시각 `·` 크기·위치 파라미터(`0.08`, `0.35`)**: text-align-2 및 유사 공문서 샘플 기준 튜닝. 다른 문서 유형에서 재조정 필요 시 후속 이슈.

## 7. 요청 사항

C안으로 단계 3 완료. 승인 시 단계 4 (통합 검증 + 최종 보고서) 진행.
