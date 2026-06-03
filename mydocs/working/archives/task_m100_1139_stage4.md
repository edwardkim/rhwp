# Stage 4 재진단 — Task #1139

## 배경

Stage 3에서 큰 둥근 괄호의 폭/패딩/선폭을 줄였지만 작업지시자가 다시 한컴오피스와 다르다고 보고했다.

## 새 후보

문24 수식 SVG 산출물에서 equation group은 다음처럼 원본 bbox 폭에 맞춰 가로 확대된다.

```svg
<g transform="translate(70.01333333333334,323.9466666666667) scale(1.1121,0.9858)">
```

즉 내부 괄호 폭을 줄이면 layout width가 작아지고, 최종 출력에서는 bbox에 맞추는 `scale_x`가 커져 glyph 전체가 다시 가로로 늘어난다. 따라서 괄호만 조정하는 방식은 한컴과의 차이를 충분히 줄이지 못한다.

## 확인할 가설

- 한컴은 Equation control의 저장 bbox를 문단 흐름의 예약 폭으로 쓰되, 수식 glyph 자체는 bbox 폭에 맞춰 강제 가로 확대하지 않는 것으로 보인다.
- rhwp는 SVG/Canvas 렌더에서 `scale_x = bbox.width / layout_box.width`를 적용해 수식 전체를 가로로 늘리므로, 문24 수식이 한컴보다 더 넓고 글리프가 왜곡되어 보일 수 있다.

## 다음 액션

1. `scale_x`를 제거하거나 `scale_y` 기준의 uniform scale로 바꾼 실험 SVG를 만든다.
2. 문24뿐 아니라 문23/문25와 우측 문27 수식 폭이 한컴 기준으로 더 가까워지는지 확인한다.
3. 시각적으로 맞으면 SVG renderer와 Canvas renderer의 Equation 배치 정책을 함께 수정한다.

## 주의

수식 control의 bbox 폭은 문단 내 다음 텍스트 위치 예약에는 계속 필요하다. glyph 렌더링 scale과 inline advance를 분리해야 한다.

