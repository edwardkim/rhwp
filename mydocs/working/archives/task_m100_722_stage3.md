# Task #722 Stage 3 단계별 보고서 — 페이지 27 본질 진단

## 진단 결과 요약

페이지 27 paragraph 779 ("Figure 4-4. 마운트된 /home과 /usr.") 결함은 정정안 E 회귀.
이전 시각 판정 (PNG 비교) 에서 baseline = fixed 라고 잘못 판단했으나, 실제 텍스트 layout 위치가 정정안 E 적용으로 인해 변경됨.

## 1. SVG 출력 비교 (baseline vs fixed)

### baseline (정정안 E 미적용)

```
y=200.97 paragraph 779 "Figure 4-4. 마운트된 /home과 /usr."
        x=56~260 (col_area 전체 폭, image 위 자유 영역)  ← PDF 정합
y=239.37 paragraph 781 "마운트는 다음과 같이..."
        x=386~ (image 우측 wrap zone)                    ← PDF 정합
```

### fixed (정정안 E 적용)

```
y=200.97 paragraph 779 "Figure 4-4. 마운트된 /home과 /usr."
        x=386~590 (image 우측 wrap zone)                ← 회귀 (PDF 위반)
y=239.37 paragraph 781 "마운트는 다음과 같이..."
        x=386~ (image 우측 wrap zone)                    ← 정합
```

baseline 의 페이지 27 layout 은 PDF 권위 자료 정합. 정정안 E 가 paragraph 779 영역에 회귀 발생.

## 2. paragraph 175 (페이지 8) vs paragraph 779 (페이지 27) IR 비교

| 항목 | paragraph 175 | paragraph 779 |
|------|--------------|--------------|
| LINE_SEG 갯수 | **2** | **1** |
| ls[0].vpos | 12960 | 8640 |
| ls[1].vpos | 14400 | (없음) |
| text_len | 34 | 30 |
| cs/sw | 24560/26464 | 24724/26300 |
| image vert_offset | 18680 | 15400 |
| 한컴 PDF layout | image 우측 wrap zone (multi-line) | image 위 자유 영역 (caption) |

두 paragraph 모두 IR cs/sw>0 wrap zone 인코딩 + 모든 LINE_SEG.vpos < image vert_offset.

**IR 단독 차이는 LINE_SEG 갯수**:
- LINE_SEG 1개 → 한컴 viewer 가 single-line caption 으로 image 위 자유 영역 layout
- LINE_SEG 2+개 → 한컴 viewer 가 multi-line wrap zone 으로 image 우측 layout

LINE_SEG 갯수가 한컴 viewer 의 layout 결과를 IR 에 인코딩한 hint 로 추정.

## 3. paragraph 데이터 (페이지 27)

```
pi=773 (image, anchor host) vpos=0
pi=774~778 (빈 paragraphs, wrap_anchors of 773) vpos=1440~7200
pi=779 (Figure 4-4 캡션 + image, anchor host) vpos=8640
pi=780 (빈 paragraph, wrap_anchors of 779) vpos=10080
pi=781 ("마운트는...", wrap_anchors of 779) vpos=11520..12960 (2 줄)
...
```

paragraph 779 의 image 가 anchor host 이고 paragraph 781 이 wrap_anchors 등록 (image 우측 wrap zone). paragraph 779 자체는 caption (image 위 자유 영역).

## 4. 본질 정정 방향 — 정정안 E 의 case 가드 추가

`src/renderer/typeset.rs:687~697` 의 anchor host self register 분기에 가드 추가:

```rust
if para.line_segs.len() >= 2 {
    st.current_column_wrap_anchors.insert(...);
}
```

- LINE_SEG ≥ 2 → 자기 등록 → wrap zone layout (paragraph 175 정합)
- LINE_SEG 1 → 자기 미등록 → col_area 전체 폭 layout (paragraph 779 정합)

## 5. Stage 4 진행 승인 요청

본 진단 결과 + Stage 4 정정 방향 (LINE_SEG ≥ 2 case 가드) 승인 요청.
