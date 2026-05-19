# Task #1001 Stage 7 — 격차 D (Shape control wrap) 진단

이슈: [#1001](https://github.com/edwardkim/rhwp/issues/1001)
계획서 v3: [`task_m100_1001.md`](../plans/task_m100_1001.md)

## 1. 진단 대상 sample

`samples/hwp3-sample16.hwp` 페이지 3 (사업개요 본문) 의 본문 박스 (Shape control):
- pi=71 paragraph 의 사각형 Shape (그라데이션/단색 fill)
- 위치 차이: HWP3 (InFrontOfText) vs HWP5 변환본 (TopAndBottom)
- 둘 다 `treat_as_char=true` (인라인 문자 취급)

## 2. SVG 직접 측정 (정밀 비교)

| 항목 | HWP3 원본 | HWP5 변환본 (Fix 후) |
|------|----------|---------------------|
| 박스 y (시작) | 152.4 px | 158.1 px |
| 박스 height | 130.2 px | 130.2 px |
| 박스 y (종료) | 282.6 px | 288.3 px |
| "2.추진방향" y | 454.6 px | 462.7 px |
| **Gap (박스 end → "2.추진방향" baseline)** | **172 px** | **174 px** |

**HWP3 와 HWP5 변환본 의 visual gap 이 거의 동일** (Δ 2 px).

## 3. 한컴 viewer 비교 (작업지시자 시각)

한컴 image 3 추정 측정:
- 박스 → "2.추진방향" gap ~ **60-80 px**

rhwp 의 gap 172-174 px 가 한컴 보다 **~100 px 더 큼**.

## 4. Shape control 속성 비교

| 속성 | HWP3 원본 | HWP5 변환본 |
|------|----------|------------|
| `treat_as_char` | true | true |
| `text_wrap` | **InFrontOfText (글앞으로)** | **TopAndBottom (위아래)** |
| Shape 크기 | 161.9×34.4 mm | 동일 |
| line_segs.line_height | 9764 HU | 9764 HU |
| 채우기 | Solid | Gradient |
| 둥근모서리 | 0% | 10% |

핵심:
- 둘 다 `treat_as_char=true` (Shape 가 인라인 문자처럼 paragraph 안에 배치)
- wrap 모드 (InFrontOfText vs TopAndBottom) 가 다르지만 시각 결과 유사 (둘 다 172-174 px gap)
- **즉 wrap 모드 차이가 visual gap 의 직접 원인 아님**

## 5. 코드 경로 분석

### 5-1. shape_reserved_heights 처리
`src/renderer/layout/shape_layout.rs:2599 calculate_shape_reserved_heights`:
```rust
// 글자처럼 취급(인라인)은 LineSeg가 이미 높이를 포함하므로 예약 불필요
if common.treat_as_char {
    continue;
}
```

`treat_as_char=true` Shape 는 shape_reserved 에서 제외. wrap=TopAndBottom 의 push-down 처리 미적용.

### 5-2. typeset 의 wrap pushdown
`src/renderer/typeset.rs:1136 pushdown_h`:
```rust
Control::Shape(s)
    if !s.common().treat_as_char  // tac=false 만 적용
        && matches!(s.common().text_wrap, TextWrap::TopAndBottom)
        ...
```

마찬가지로 `treat_as_char=true` 제외.

### 5-3. 결론

`treat_as_char=true` Shape 의 처리에 wrap=TopAndBottom specific 한 padding 추가는 없음. 그러나 **paragraph 의 line_height (lh) = Shape height (9764 HU = 130 px)** 가 paragraph y_offset 누적에 포함됨 (typeset.rs:619, 879, 905 등).

## 6. Gap 누적 추정 (이론값 vs 실측)

paragraph 누적 (HWP5 변환본, dump-pages 기반):

| Step | 항목 | 누적값 |
|------|------|--------|
| Box 시작 | pi=71 vpos (line_segs) | 158 px |
| Box 영역 | + Shape height (130 px) | 288 px (= 박스 end) |
| pi=71 line_spacing | + ls (~10 px) | 298 px |
| pi=72 sb + h | + 3.8 + 7.8 | 309.6 px |
| pi=73 sb | + 11.4 | 321 px |
| pi=73 baseline | + ~13.6 (ascent) | 334.6 px |

**이론값**: "2.추진방향" baseline = **~334.6 px**
**실측값** (SVG): **462.7 px**
**격차**: **~128 px** — 정확히 **Shape height (130 px)**

## 7. Root cause 가설

**Shape height (130 px) 가 paragraph y_offset 에 두 번 카운트** (paragraph 의 line_height 와 별도로 한 번 더 추가).

가능한 원인:
1. `treat_as_char=true` Shape 의 line_height 포함 + 별도 wrap pushdown
2. Shape paragraph 의 rendered height 가 두 번 합산되는 layout 경로
3. line_segs.vertical_pos 와 paragraph 누적 y_offset 중복 사용

## 8. Fix 후보 (Stage 9)

### 후보 D1 — treat_as_char Shape 의 wrap=TopAndBottom 처리 stripped
- `treat_as_char=true` 인 Shape 의 wrap 을 InFrontOfText 로 effectively 무시
- 이미 inline 으로 처리 중이므로 wrap 무관해야 정합

### 후보 D2 — paragraph y_offset 누적 시 Shape inline 영향 deduplicate
- line_height 가 이미 Shape height 포함 시, 별도 pushdown 없음 보장
- 다른 path (composer/pagination 등) 에서 double count 차단

### 후보 D3 — line_segs.vpos 기반 y 계산으로 통일
- 누적 ParaShape spacing 대신 line_segs.vertical_pos 직접 사용
- 다른 paragraph 의 vpos 계산도 검증 필요

### 후보 D4 — Shape paragraph 의 line_height 무시 (text height 만 사용)
- treat_as_char + wrap=TopAndBottom Shape paragraph 는 line_height 가 Shape height 가 아닌 text 만 반영
- Shape 는 별도 layer 로 그리고 paragraph 영역 차지 안 함

## 9. Risk 평가

격차 D fix 는 모든 Shape control (HWP3, HWP5, HWPX, 변환본) 에 영향. Sweep 회귀 critical:
- aift.hwp (Shape 다수)
- sample16/17/19 등 (그림 + Shape)
- 시험지 (다이어그램 + 표)
- HWPX 변환본
- 일반 HWP5

## 10. 잔존 / 관련 이슈

- 격차 E (HWP3 spacing 단위) 는 격차 D 와 별개 이슈 — Stage 8 별도 진단
- Shape control 처리 변경은 본 Task 의 fix 가 아닌 후속 Task (예: Task #1002) 로 분리 가능 — 작업지시자 결정 필요

## 11. Stage 7 산출물

- 본 보고서: `mydocs/working/task_m100_1001_stage7.md`
- 진단 결론: rhwp 의 Shape paragraph y_offset 누적에 ~Shape height 만큼 double count 발생 (정확한 위치 추가 추적 필요)

## 12. Stage 8 진입 결정

다음 단계 선택:
1. Stage 8 (HWP3 spacing 단위) 진단 진행
2. Stage 9 (Fix 적용) — 후보 D1 ~ D4 평가 후 선정
3. **본 Task #1001 의 fix 범위 재검토** — 격차 D/E 가 본 Task 의 핵심 (HWP5 변환본 fix) 과 별개의 일반 rendering 이슈일 가능성
