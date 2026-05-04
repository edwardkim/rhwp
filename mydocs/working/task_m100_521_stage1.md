# Task #521 Stage 1 — 정밀 진단 보고서

**날짜**: 2026-05-04
**브랜치**: `pr-task521` (devel `f807378a`)
**이슈**: [#521](https://github.com/edwardkim/rhwp/issues/521) — exam_eng p2 18번 답안지 위치 박스 하단 너무 가까움

## 1. 본 devel 측정 (현 발현 상태)

### 1.1 18번 문제 우측 단 측정

| 항목 | 위치 | 비고 |
|------|------|------|
| 18번 문제 헤더 "18.- 다음 글의..." | y=228.84 | x=582.05 (col 1 시작) |
| 박스 (table border rect) top | y=243.59 | x=597.12, w=408.19, h=288.09 |
| 박스 bottom | y=243.59 + 288.09 = **531.68** | — |
| ① 첫 답안 | y=**543.95** | 박스 bottom + 12.27 px gap |
| ② / ③ 답안 | y=568.45 / 588.37 | 줄간격 24.50 / 19.92 px |

### 1.2 박스 ↔ ① 여백 측정

- 본 devel: **gap = 12.27 px** (박스 bottom 531.68 → ① 543.95)
- 이슈 등록 시점 (1684×2382 스케일): SVG gap ≈ 8.0 px / 1.50 = 5.33 px
- PDF 한컴 2010: gap ≈ 30 px / 1.50 = 20 px

→ **본 devel 의 gap 12.27 px** 가 이슈 등록 시점 5.33 px 보다 개선됐으나 PDF 20 px 와는 여전히 -7.7 px 차이.

## 2. 이슈 본문 가설 검증

### 2.1 가설: BehindText 그림 ctrl[1] 의 vertical extent 가 paragraph height 미반영

**검증**:
- ctrl[0] 그림 BehindText, offset 0, h=67.8mm = 19207 HU
- ctrl[1] 그림 BehindText, offset 58.1mm = 16469 HU, h=18.5mm = 5250 HU
- ctrl[1] paragraph-relative bottom = 16469 + 5250 = **21719 HU**
- ctrl[2] 표 (TopAndBottom, tac=true) h=76.2mm = 21607 HU + outer_margin_bottom 600 = **22207 HU**

→ **ctrl[1] bottom (21719) < ctrl[2] bottom (22207)**. 즉 ctrl[1] 그림이 표 안에 수용됨. paragraph height 결정에 ctrl[1] 미반영도 문제 없음.

→ **이슈 본문의 가설은 본 devel 코드 기준으로 재해석 필요**. ctrl[1] 가 paragraph height 를 연장시켜야 한다는 가정은 필요 없음.

### 2.2 IR vpos 정합 측정

```
pi=104 vpos=2254, ls[0] lh=22207
pi=104 종료 (vpos+lh): 2254 + 22207 = 24461 HU
pi=105 IR vpos: 24805 HU
gap (IR 기준): 24805 - 24461 = 344 HU = 4.59 px
```

→ IR 의 pi=104 → pi=105 gap 은 **4.59 px** (정상 trailing line_spacing).

### 2.3 layout.rs:1403-1432 prev_has_overlay_shape 가드

본 devel 코드는 prev_pi 가 BehindText/InFrontOfText/TopAndBottom-Para overlay shape 를 갖는 경우 **vpos correction 을 skip**:

```rust
let prev_has_overlay_shape = paragraphs.get(prev_pi).map(|p| {
    p.controls.iter().any(|c| match c {
        Control::Picture(pic) => {
            ...
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText) || ...
        }
        ...
    })
}).unwrap_or(false);
if !prev_has_overlay_shape { ... vpos correction ... }
```

→ pi=104 의 ctrl[0]/[1] 이 BehindText 이므로 `prev_has_overlay_shape = true`. pi=105 시작 시 **vpos correction skip**, sequential y_offset 사용.

## 3. 추가 발견 — 더 큰 결함 (이슈 본문 외)

### 3.1 박스 (표 ctrl[2]) 셀 텍스트 미렌더링

pi=104 의 표 (1×1) 셀에 5 paragraphs (총 ~620 chars 이메일 본문) 이 있으나 **SVG 에 본문 텍스트 0개**:

```
SVG 페이지 2 우측 단 y 240-540 영역 <text> 요소: 0
포함 내용: rect (테두리), <image> (브라우저 프레임 그림)
미포함: 셀 paragraphs[0..5] 의 텍스트 ("Dear Rosydale...", "We are really grateful...", "Sincerely, Martha Kingsley, Race Manager")
```

**8 페이지 전체 sweep**: "Dear/Sincerely/Race/Marathon/Martha" 키워드 매칭 **0 회**. 이메일 본문이 어떤 페이지에도 렌더링 안 됨.

### 3.2 영향

- 박스 자체는 그려지나 **내용 누락** (시각적으로 빈 박스 + 브라우저 프레임 이미지)
- ① 답안 위치 -7.7 px 시프트는 **별개 결함** — pagination 의 paragraph height 산출에 셀 미렌더 영향 가능성

## 4. 가능 본질 후보 (재진단)

| 후보 | 영역 | 본질 |
|------|------|------|
| **A** | `paragraph_layout.rs` 또는 `table_layout.rs` 셀 paragraph 렌더 | 셀 paragraphs (5건) 가 SVG `<text>` 로 출력 안 됨 |
| **B** | `pagination/engine.rs` paragraph height 누적 | pi=104 의 height 가 셀 미렌더로 잘못 계산되어 pi=105 위치 영향 |
| **C** | `typeset.rs::calc_paragraph_height` BehindText | 이슈 본문 가설 — 본 devel 에서 부분 발현 (-7.7 px) |
| **D** | `composer.rs` 셀 paragraph compose | tac=true 인 표의 셀 안 paragraph compose 누락 |

→ **A/D 가 가장 가능성 높음**. 셀 본문 텍스트가 SVG 에 안 나오는 게 본질이고, ① 위치 시프트는 그 결과의 부수 영향일 가능성.

## 5. 영향 범위 추정

이슈 본문은 pi=104 같은 "BehindText 그림 + 인라인 표 + 다음 문단" 패턴만 거론. 실제 영향:

- **셀 본문 텍스트 미렌더링** 은 더 광범위할 가능성 — `tac=true` 인 표의 셀 paragraph 처리 영역
- exam_eng 외 다른 fixture 에서 동일 결함 발현 여부 추가 sweep 필요

## 6. 옵션 평가 (재진단 후)

### 6.1 옵션 A — 셀 본문 텍스트 미렌더링 정정 (root cause 가능성)

- 영향 범위: 가능 가시 결함 다수 (다른 fixture 영향 sweep 필요)
- 회귀 위험: 큼 (셀 paragraph 렌더 로직 변경)
- 부수 효과: ① 위치 자동 정합 가능성

### 6.2 옵션 B — paragraph height 만 정정 (이슈 본문 가설)

- 영향 범위: BehindText overlay paragraph 모두
- 회귀 위험: 큼 (`feedback_essential_fix_regression_risk` — overlay shape 처리)
- 부수 효과: 셀 본문 미렌더링 미해결

### 6.3 옵션 C — Stage 2 추가 진단 (옵션 A vs B 결정)

`tac=true` 표 (`is_tac_table_inline`) 의 셀 본문 렌더링 코드 경로 추적.
- `composer.rs` 셀 paragraph compose 확인
- `table_layout.rs` 셀 paragraph 렌더링 경로 추적
- 다른 fixture 의 `tac=true` 표 셀 본문 렌더 정상 여부 sweep

→ 옵션 C 권장 (옵션 결정 전 추가 진단).

## 7. 광범위 sweep 후보

`tac=true` 인 표 + 셀 paragraph 가 있는 fixture 식별:

```bash
for f in samples/*.hwp; do
  hits=$(./target/release/rhwp dump "$f" 2>/dev/null | grep -c "tac=true.*paras=[1-9]")
  [ "$hits" -gt 0 ] && echo "$hits : $f"
done | sort -rn | head -10
```

(Stage 2 진행 시 실행)

## 8. 작업지시자 결정 사항

**Stage 1 진단 결과 종합**:

1. 이슈 본문 가설 (BehindText vertical extent 미반영) 은 본 devel 코드 기준 재해석 필요
2. **추가 발견**: pi=104 박스 셀 본문 텍스트 (이메일) 가 SVG 에 미렌더링 — 이슈 본문보다 더 큰 결함
3. ① 위치 -7.7 px 시프트 (이슈 등록 시점 -20 px → 부분 개선) 는 셀 미렌더의 부수 영향일 가능성

**진행 결정 후보**:

- **D1**: Stage 2 추가 진단 (옵션 C) — 셀 paragraph 미렌더링 root cause 추적 + 다른 fixture sweep
- **D2**: 이슈 #521 의 가설 (옵션 B) 만 진행 — 본 발견 무시
- **D3**: 이슈 #521 본문 갱신 — 새 발견 반영 (셀 미렌더링 이 root cause) + Stage 2 추가 진단 → 결정
- **D4**: 우선순위 재평가 보류 — 더 시급한 결함 검토

권장: **D3 + D1** (이슈 갱신 후 Stage 2 진행). 진행 결정 부탁드립니다.
