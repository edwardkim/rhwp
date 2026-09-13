# issue 7018 fixture

자리차지(`treat_as_char`) 표를 **글자 끝에 앵커한 호스트 문단**의 최소 재현체다.

2쪽 문단 27 은 텍스트 `  마. 행정박물류` 뒤에 표를 달고, 한/글은 저장 사다리를 **두 줄**로 낸다.

```text
  seg[0] vpos=39764 lh=1200  th=1200  bl=1020   textpos=0    ← 글자 줄
  seg[1] vpos=41924 lh=15792 th=15792 bl=13423  textpos=11   ← 표 줄 (본문 끝에서 시작)
```

`textpos=11` 이 본문 길이와 같다 = 표가 줄 하나를 통째로 가졌다는 한/글의 기록이다.
두 줄에 본문 시작 오프셋(75.6px)을 더하면 글자 줄 상단 605.8px · baseline 619.4px 이고,
이 baseline 이 기준 PDF 의 잉크 구간(605.5..620.5)과 맞는다. 표는 636.4px 에 놓이며 이 수정
전후로 움직이지 않는다.

종전에는 글자 런이 **표 줄의 baseline(179.0px)** 을 받아 baseline 이 784.8px 에 찍혔다
(165.4px 하강, 표 둘째 행 글자와 겹침). `layout_inline_table_paragraph` 가 `table_seg` 와
`text_seg` 를 이미 구분해 놓고도 `wrapped_below_table` 이 아닐 때 표 줄 값을 쓴 탓이다.

기준 PDF 는 문서 저장 제품(한컴 2020)과 같은 엔진으로 새로 산출했다.
