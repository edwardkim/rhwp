# Task #321 Stage 2 — 문단간 vpos-reset 강제 분할

## 작업 내용

`src/renderer/typeset.rs::typeset_section`의 문단 순회 루프에 다음 규칙 추가:

- `para_idx > 0 && !st.current_items.is_empty()`
- `curr_first_vpos == 0 && prev_last_vpos > 5000 HU (≈ 1.76mm)`
- 위 조건 만족 시 `st.advance_column_or_new_page()` 호출

즉, HWP LINE_SEG가 pi 경계에서 vpos를 0으로 리셋한 경우(HWP 원본이 해당 위치에서 페이지/단 분할을 의도한 경우) 현재 단을 flush하고 다음 단/페이지로 넘어간다.

## 21_언어 효과

### Before (Stage 1)

```
페이지 1 단 1 (items=22):
  PartialParagraph pi=9 lines=11..14
  FullParagraph pi=10..pi=30  ← pi=30 vpos=0 무시하고 col 1에 쌓음
  LAYOUT_OVERFLOW: page=0, col=1, para=28/29/30, overflow=9.5px
  pi=29와 pi=30 동일 y=1421.5 에 클램프 → 텍스트 겹침
```

### After (Stage 2)

```
페이지 1 단 1 (items=21):
  PartialParagraph pi=9 lines=11..14
  FullParagraph pi=10..pi=29  ← pi=30은 pi=29 직후 vpos=0 감지로 page 2로 이동
  LAYOUT_OVERFLOW: page=0, col=1, para=28/29, overflow=9.5px
  pi=28 y=1420.3, pi=29 y=1434.0 (겹침 해소)

페이지 2 단 0 (items=14, used=841.3, hwp_used≈848.1, diff=-6.8):
  FullParagraph pi=30  ← HWP 원본 의도 위치
  FullParagraph pi=31, pi=32..pi=43
```

### 시각 검증

- pi=29의 ③이 y=1433.97(font-size=14.67)로 렌더, pi=28의 ②가 y=1420.28. 두 문자의 수직 범위가 1px 이상 분리되어 **시각적 텍스트 겹침 해소**.
- pi=30이 페이지 2 상단으로 이동하여 HWP 원본과 동일한 위치에 렌더.

### 잔존 LAYOUT_OVERFLOW

- pi=28/pi=29 각 9.5px 초과 경고는 여전히 출력되나 실제 body text는 col_bottom(1436.2) 이내에 렌더됨. 원인: 포맷터의 trailing line_spacing 포함 계산과 layout의 vpos 기반 진행 사이 드리프트. Stage 2 범위 외.

## 4 샘플 회귀

| 샘플 | Before | After | 비고 |
|------|--------|-------|------|
| 21_언어 | 15 | 15 | pi=30 이동, 페이지 수 불변 |
| exam_math | 20 | 20 | 변동 없음 |
| exam_kor | 24 | 24 | 기존 2건 overflow 유지 |
| exam_eng | 9 | 9 | 기존 2건 overflow 유지 |

`cargo test --release`: 992 passed, 0 failed.

## Task #311과의 차이

Task #311은 Paginator 경로에서 유사한 vpos-reset 강제 분할을 시도했으나 21_언어 19→20쪽 회귀. 본 Task #321은 TypesetEngine 경로에서 적용하였고 페이지 수 불변을 달성. 원인 추정:

- TypesetEngine이 column 가용 공간을 더 정확히 쓰기 때문에 분할 1회로 인한 페이지 증가가 없음.
- inter-paragraph reset만 대상(intra-paragraph reset은 기존 `detect_column_breaks_in_paragraph`가 처리).

## 결정사항

Stage 1 진단 훅(`RHWP_TYPESET_DRIFT`)은 추후 디버깅 가치가 있어 보존.
