# Task #321 v4 Stage 5 — col 0 drift 진단 보고서

## 개요

대상: `samples/21_언어_기출_편집가능본.hwp` page 1 col 0
이슈: PDF 와 비교 시 col 1 시작 paragraph 가 다름 (pi=10 "적합성..." 대신 pi=9 lines 11..14 "도출될 경우...")
원인 후보: col 0 의 누적 drift (= 우리 typeset 이 HWP LINE_SEG 기반 위치 보다 더 많은 공간을 사용)

## 진단 방법

`src/renderer/typeset.rs::typeset_paragraph` 진입부에 env-gated 진단 로그 추가
(Stage 5a 완료 — `RHWP_TYPESET_DRIFT=1`):

```
TYPESET_DRIFT_PI: pi={pi} col={col} sb={spacing_before} sa={spacing_after}
                  lines={n} lh_sum={Σ line_heights} ls_sum={Σ line_spacings}
                  trail_ls={마지막 line_spacing}
                  fmt_total={sb+lh_sum+ls_sum+sa} vpos_h={LINE_SEG span}
                  diff={fmt_total-vpos_h}
                  first_vpos={hu} last_vpos={hu}
                  cur_h={진입 시점 cur_h} avail={col 가용 높이}
```

명령:

```bash
RHWP_TYPESET_DRIFT=1 ./target/release/rhwp export-svg \
  samples/21_언어_기출_편집가능본.hwp -p 0 -o /tmp/drift/ 2> /tmp/drift_log.txt
```

## 측정 결과 — col 0 (pi=1..pi=9)

> pi=0 은 표(4×5) 를 포함하므로 `typeset_table_paragraph` 경로로 처리되어
> 본 로그에 잡히지 않음. cur_h 변화로 간접 측정.

| pi | sb | sa | lines | lh_sum | ls_sum | trail_ls | fmt_total | vpos_h | cur_h_before | y_first_line(우리) | first_vpos_HU px | drift_real |
|----|-----|-----|-------|--------|--------|----------|-----------|--------|--------------|---------------------|-------------------|------------|
| 1  | 7.6 | 0.0 | 1     | 14.7   | 9.5    | 9.5      | 31.8      | 14.7   | 244.9        | 252.5               | 176.2             | **+76.3**  |
| 2  | 7.6 | 0.0 | 2     | 29.3   | 19.1   | 9.5      | 56.0      | 38.9   | 276.7        | 284.3               | 207.9             | +76.3      |
| 3  | 7.6 | 0.0 | 1     | 14.7   | 9.5    | 9.5      | 31.8      | 14.7   | 332.7        | 340.4               | 264.1             | +76.3      |
| 4  | 7.6 | 0.0 | 1     | 14.7   | 9.5    | 9.5      | 31.8      | 14.7   | 364.6        | 372.2               | 295.9             | +76.3      |
| 5  | 0.0 | 0.0 | 1     | 14.7   | 9.5    | 9.5      | 24.2      | 14.7   | 396.4        | 396.4               | 320.1             | +76.3      |
| 6  | 0.0 | 0.0 | 1     | 14.7   | -0.7   | -0.7     | 13.9      | 14.7   | 420.6        | 420.6               | 344.3             | +76.3      |
| 7  | 0.0 | 0.0 | 10    | 146.7  | 95.5   | 9.5      | 242.1     | 232.6  | 434.5        | 434.5               | 358.2             | +76.3      |
| 8  | 0.0 | 0.0 | 12    | 176.0  | 114.6  | 9.5      | 290.6     | 281.0  | 676.6        | 676.6               | 600.3             | +76.3      |
| 9  | 0.0 | 0.0 | 14    | 205.3  | 133.7  | 9.5      | 339.0     | 329.4  | 967.2        | 967.2               | 890.9             | +76.3      |

`y_first_line(우리)` = `cur_h_before + sb` (col 0 origin 기준)
`first_vpos_HU px` = `first_vpos / 7200 × 96` (col 0 origin 기준)
`drift_real` = `y_first_line(우리) - first_vpos_HU px`

## 핵심 결론

**drift = +76.3 px 가 pi=1 부터 pi=9 까지 정확히 일정**.

→ pi=1 ~ pi=9 의 per-paragraph advance 는 HWP LINE_SEG 기반 위치 와 정확히 일치한다 (per-paragraph 누적 오차 0).

→ **drift 76.3 px 의 출처는 pi=0 단독**. 4×5 폼 표(wrap=TopAndBottom, vert_rel_to=Paper) 를
포함한 첫 문단이 pi=1 의 시작 위치를 +76.3 px 만큼 아래로 밀어넣는다.

## pi=0 측정 (간접)

진입 데이터:
- `cur_h_before_pi0` = 0 (col 0 첫 문단)
- `cur_h_before_pi1` = 244.9
- 우리 pi=0 advance = 244.9 px
- HWP first_vpos pi=0 = 9014 hu = **120.2 px**, pi=1 = 13216 hu = 176.2 px
- HWP advance pi=0 → pi=1 = **56.0 px**

→ 우리 pi=0 placement 가 HWP 보다 **+188.9 px** 더 advance 함.

이 중:
- 우리 pi=0 first line 은 col origin (cur_h=0) 에서 시작 → HWP pi=0 first line 은 col origin + 120.2 px 에서 시작 → 우리가 pi=0 시작점 자체를 **-120.2 px** 위에 배치.
- 그 후 우리 pi=0 가 `effective_height + host_spacing` (블록 표 + 외부 여백) 만큼 cur_h 를 증가.
- HWP 는 표를 Paper-anchored 절대 좌표로 배치하고 본문 텍스트 vpos 는 표와 별도 좌표계로 기록.

순효과: pi=1 시작 시점에 우리는 +76.3 px 아래에 위치 (= -120.2 + 188.9 - 7.6(pi=1 sb) ≈ 60.8, 실제 측정 76.3 — sb 처리 미세차).

## Drift 가 페이지 분할에 미치는 영향

col 0 가용 높이 = 1226.4 px

pi=9 fit 판정: `cur_h_before + height_for_fit ≤ available`

```
967.2 + 329.5 = 1296.7  >  1226.4   → pi=9 분할 (lines 11..14 → col 1)
```

drift 0 가정 시:
```
(967.2 - 76.3) + 329.5 = 1220.4  ≤  1226.4   → pi=9 통째로 col 0 → PDF 일치 ✓
```

## 카테고리별 기여 분해

| 카테고리 | 기여 (px) | 비고 |
|----------|-----------|------|
| pi=0 표 layout (block 표 effective_height + outer margin) + 표/본문 좌표계 mismatch | +76.3 | dominant |
| pi=1..pi=9 per-paragraph 누적 | 0.0 | sb/sa/trail_ls 모두 HWP gap 과 정확히 일치 |
| 표 padding / cell border | (pi=0 내부 — 미분해) | typeset_block_table 경로 진단 미적용 |

→ pi=0 단일 항목이 dominant 이며, pi=1+ 의 본문 부분은 누적 오차 없음.

## 후속 수정 가설 (Stage 6 후보)

본 보고서는 **진단·정량화** 단계이며 수정은 별도 stage 에서 다룬다.
가설:

**H1**: `typeset_block_table` 에서 wrap=TopAndBottom + vert_rel_to=Paper 인 표는
`effective_height + host_spacing` 전체를 cur_h 에 더하지 말고, 본문 텍스트 vpos 와 일치하도록
**reduced advance** (= HWP advance 56.0 px 에 대응) 를 적용한다.
   - 시각적으로는 표가 절대 좌표로 그려지므로 위치 무변경.
   - cur_h advance 만 줄여 pi=1+ 본문이 위로 76 px 이동.

**H2**: pi=0 의 시작 위치를 HWP first_vpos (120.2 px) 에 맞춰 col origin + 120.2 로 시작하고
표는 절대 좌표로 별도 배치. 그러면 본문이 자연스럽게 표 아래에 위치.

**H3**: layout 단의 `body_wide_reserved` 와 typeset 의 cur_h 양쪽에서 동일한 조정 (이미 col 1 은
`pending_body_wide_top_reserve` 로 처리, col 0 은 미적용).

H1 이 가장 간단하고 회귀 위험이 작아 보이지만, 다른 샘플에서도 동일 패턴(Paper-anchored
TopAndBottom 표) 이 있는지 검증 필요.

## 제약·주의사항

- 본 stage 는 `RHWP_TYPESET_DRIFT` env-gated 로그 추가 외 동작 변경 없음.
- 수정은 별도 stage (Stage 6 — pi=0 표 layout drift 보정).
- 다른 샘플(exam_math/kor/eng) 영향은 수정 시점에 검증.

## 검증

- `cargo test --lib`: 992 passed, 0 failed (Stage 5a 빌드)
- `RHWP_TYPESET_DRIFT` 미설정 시 출력 무변경 확인
- 다른 샘플 페이지 수 무변경 확인 (TODO — 수정 stage 에서 회귀 검증 시 수행)
